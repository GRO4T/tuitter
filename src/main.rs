use clap::Parser;
use std::error;
use std::net::{Shutdown, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
use messages::{JoinChatRequest, JoinChatResponse, TuitterMessage};
use protobuf::Message;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

const SERVER_ADDR: &str = "localhost:8081";
const UDP_DATAGRAM_SIZE: usize = 1024;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Username
    #[arg(short, long, default_value = "")]
    user: String,

    /// Run in server mode
    #[arg(short, long, action)]
    server: bool,
}

struct Client {
    ip_addr: String,
}

fn handle_server() -> Result<()> {
    let clients: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));

    let clients_clone = Arc::clone(&clients);
    let conn_listener = thread::spawn(move || {
        let listener = TcpListener::bind(SERVER_ADDR).unwrap();
        log::info!("Server listening on {SERVER_ADDR}");
        let mut next_port = 8083;
        for stream in listener.incoming() {
            log::debug!("New connection");
            let mut stream = stream.unwrap();

            // Read connection request
            let join_chat_request = JoinChatRequest::parse_from_reader(&mut stream);
            let join_chat_request = join_chat_request.unwrap();
            log::debug!("{} joined the chat", join_chat_request.username);

            // Send connection response
            let mut join_chat_response = JoinChatResponse::new();
            join_chat_response.status = 200;
            join_chat_response.port = next_port;
            join_chat_response.write_to_writer(&mut stream).unwrap();
            stream.shutdown(Shutdown::Write).unwrap();

            // Save new client data
            let mut clients = clients_clone.lock().unwrap();
            (*clients).push(Client {
                ip_addr: format!("localhost:{next_port}"),
            });

            next_port += 1;
        }
    });

    let clients_clone = Arc::clone(&clients);
    let msg_broadcaster = thread::spawn(move || {
        let socket = UdpSocket::bind("localhost:8082").unwrap();

        loop {
            thread::sleep(time::Duration::from_millis(1000));
            let clients = clients_clone.lock().unwrap();
            for client in (*clients).iter() {
                let mut message = TuitterMessage::new();
                message.sender = String::from("server");
                message.text = String::from("Hello everyone");
                let datagram = message.write_to_bytes().unwrap();

                if datagram.len() > UDP_DATAGRAM_SIZE {
                    panic!("UPD datagram size exceeded! ({})", datagram.len())
                }

                socket
                    .send_to(&datagram[..], client.ip_addr.as_str())
                    .unwrap();
            }
        }
    });

    conn_listener.join().unwrap();
    msg_broadcaster.join().unwrap();

    Ok(())
}

fn handle_client(username: &str) -> Result<()> {
    let mut stream = TcpStream::connect(SERVER_ADDR)?;

    // Join the chat
    let mut join_chat_request = JoinChatRequest::new();
    join_chat_request.username = username.to_owned();
    join_chat_request.write_to_writer(&mut stream)?;
    stream.shutdown(Shutdown::Write)?;

    // Wait for server response
    let join_chat_response = JoinChatResponse::parse_from_reader(&mut stream)?;

    // Listen for messages
    let ip_addr = format!("localhost:{}", join_chat_response.port);
    let socket = UdpSocket::bind(ip_addr)?;
    loop {
        let mut buf = [0; UDP_DATAGRAM_SIZE];
        let (number_of_bytes, _) = socket.recv_from(&mut buf).unwrap();
        let msg = TuitterMessage::parse_from_bytes(&buf[..number_of_bytes])?;
        println!("{}: {}", msg.sender, msg.text);
    }
}

fn main() -> Result<()> {
    let mut request = JoinChatRequest::new();
    request.username = "Bob".to_string();

    env_logger::init();
    let args = Args::parse();
    if args.server {
        log::info!("TUItter server started");
        handle_server()?;
    } else if !args.user.is_empty() {
        log::info!("TUItter client started (username={})", args.user);
        handle_client(&args.user)?;
    } else {
        panic!("User not specified!");
    }
    Ok(())
}
