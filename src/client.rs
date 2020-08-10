use std::io::prelude::*;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;


pub enum PayloadSignal {
    MessageSignal,
    InterruptSignal,
}

pub type ClientPayload = (usize, SocketAddr, String, PayloadSignal);


pub struct Client {
    pub client_id: usize,
    pub stream: TcpStream,
    pub socket_addr: SocketAddr,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Client {
    pub fn new(client_id: usize, stream: TcpStream, socket_addr: SocketAddr, sender: mpsc::Sender<ClientPayload>) -> Client {
        println!("New connection from {} established.", socket_addr);

        let mut stream_clone = stream.try_clone().expect(&format!("Could not clone stream for client: {}", client_id));
        let socket_addr_clone = socket_addr.clone();

        let thread = thread::spawn(move || loop {
            let buf = &mut [0u8; 16];

            if let Ok(recv_bytes) = stream_clone.read(buf) {
                if recv_bytes == 0 {
                    println!("Client {} disconnected from the server.", socket_addr_clone);
                    // TODO: Flag for despawn this client and join the thread
                    sender.send((client_id, socket_addr_clone, String::new(), PayloadSignal::InterruptSignal)).ok();
                    break;
                }

                let message = String::from_utf8_lossy(buf);
                let pat: &[_] = &['\x00', '\x0A', '\x0D'];
                let message = format!("{}: {}", socket_addr_clone, message.trim_matches(pat));

                sender.send((client_id, socket_addr_clone, message, PayloadSignal::MessageSignal))
                    .expect(&format!("Client {}@{} could not send payload to the downstream!", client_id, socket_addr_clone));
            }
        });

        Client { client_id, stream, socket_addr, thread: Some(thread) }
    }
}