use clap::Parser;
use env_logger;
use log;
use std::error;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

const SERVER_ADDR: &str = "localhost:8081";

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
            let stream = stream.unwrap();
            let mut buf_reader = BufReader::new(&stream);
            let mut msg = String::new();
            buf_reader.read_line(&mut msg).unwrap();
            if msg.trim() == "ping" {
                let mut buf_writer = BufWriter::new(&stream);
                buf_writer.write(next_port.to_string().as_bytes()).unwrap();
                buf_writer.flush().unwrap();
                let mut clients = clients_clone.lock().unwrap();
                (*clients).push(Client {
                    ip_addr: format!("localhost:{next_port}"),
                });
                next_port += 1;
            }
        }
    });

    let clients_clone = Arc::clone(&clients);
    let msg_broadcaster = thread::spawn(move || {
        let socket = UdpSocket::bind("localhost:8082").unwrap();

        loop {
            thread::sleep(time::Duration::from_millis(1000));
            let clients = clients_clone.lock().unwrap();
            for client in (*clients).iter() {
                log::debug!("Sending `Hello` to {}", client.ip_addr);
                socket
                    .send_to("Hello".as_bytes(), client.ip_addr.as_str())
                    .unwrap();
            }
        }
    });

    conn_listener.join().unwrap();
    msg_broadcaster.join().unwrap();

    Ok(())
}

fn handle_client() -> Result<()> {
    let stream = TcpStream::connect(SERVER_ADDR)?;

    let mut buf_writer = BufWriter::new(&stream);
    let request = "ping\n";
    buf_writer.write(request.as_bytes())?;
    buf_writer.flush()?;

    let mut buf_reader = BufReader::new(&stream);
    let mut response = String::new();
    buf_reader.read_to_string(&mut response)?;
    let udp_port: u32 = response.parse()?;

    log::info!("Listening on localhost:{udp_port}");
    let socket = UdpSocket::bind(format!("localhost:{udp_port}")).unwrap();

    loop {
        let mut buf = [0; 10];
        socket.recv_from(&mut buf).unwrap();
        let msg = String::from_utf8(buf.to_vec()).unwrap();
        log::debug!("Received: {msg}");
    }
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    if args.server {
        log::info!("TUItter server started");
        handle_server()?;
    } else if args.user != "" {
        log::info!("{} joined the chat", args.user);
        handle_client()?;
    } else {
        panic!("User not specified!");
    }
    Ok(())
}
