use std::io::prelude::*;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

pub enum PayloadSignal {
    MessageSignal,
    InterruptSignal,
}

pub type ClientPayload = (SocketAddr, PayloadSignal, Option<String>);

pub struct Client {
    pub stream: TcpStream,
    pub socket_addr: SocketAddr,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Client {
    pub fn new(
        mut stream: TcpStream,
        socket_addr: SocketAddr,
        sender: mpsc::Sender<ClientPayload>,
    ) -> Client {
        println!("New connection from {} established.", socket_addr);

        let stream_clone = stream.try_clone().expect(&format!(
            "Could not clone stream for client: {}",
            socket_addr
        ));
        let socket_addr_clone = socket_addr.clone();

        let thread = thread::spawn(move || loop {
            let mut buf = [0u8; 32];

            if let Ok(recv_bytes) = stream.read(&mut buf) {
                // TODO: Prevent message that higher than the buffer will be defragmented
                if recv_bytes == 0 {
                    println!("Client {} disconnected from the server.", socket_addr_clone);
                    sender
                        .send((socket_addr_clone, PayloadSignal::InterruptSignal, None))
                        .ok();
                    break;
                }

                let buf = buf
                    .iter()
                    .filter_map(|x| {
                        if *x >= 0x20 && *x <= 0x7E {
                            return Some(*x);
                        }

                        None
                    })
                    .collect::<Vec<u8>>();

                let message = String::from_utf8(buf)
                    .expect("Could not parsing buffer into valid UTF-8!")
                    .trim()
                    .to_string();

                /* No need to process empty buffer */
                if message.len() <= 0 {
                    continue;
                }

                sender
                    .send((
                        socket_addr_clone,
                        PayloadSignal::MessageSignal,
                        Some(message),
                    ))
                    .expect(&format!(
                        "Client {} could not send payload to the downstream!",
                        socket_addr_clone
                    ));
            }
        });

        Client {
            stream: stream_clone,
            socket_addr,
            thread: Some(thread),
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.stream.shutdown(Shutdown::Both).expect(&format!(
            "Could not shutdown the stream of client: {}",
            self.socket_addr
        ));

        if let Some(thread) = self.thread.take() {
            thread.join().expect(&format!(
                "Trying to terminate thread of client {} but failed.",
                self.socket_addr
            ));
        }

        println!("Connection with {} successfully closed!", self.socket_addr);
    }
}
