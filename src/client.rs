use std::io::prelude::*;
use std::io::BufReader;
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
    pub max_buffer: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Client {
    pub fn new(
        stream: TcpStream,
        socket_addr: SocketAddr,
        max_buffer: usize,
        sender: mpsc::Sender<ClientPayload>,
    ) -> Client {
        println!("New connection from {} established.", socket_addr);

        let stream_clone = stream.try_clone().expect(&format!(
            "Could not clone stream for client: {}",
            socket_addr
        ));
        let socket_addr_clone = socket_addr.clone();
        let mut reader = BufReader::with_capacity(max_buffer, stream);

        let thread = thread::spawn(move || loop {
            let mut buffer = String::new();

            match reader.read_line(&mut buffer) {
                Ok(recv_bytes) => {
                    if recv_bytes == 0 {
                        println!("Client {} disconnected from the server.", socket_addr_clone);
                        sender
                            .send((socket_addr_clone, PayloadSignal::InterruptSignal, None))
                            .ok();
                        break;
                    }

                    /* Do nothing when the buffer is too large from current maximum */
                    if recv_bytes >= max_buffer {
                        continue;
                    }

                    let message = buffer
                        .as_bytes()
                        .iter()
                        .filter_map(|x| {
                            if *x >= 0x20 && *x <= 0x7E {
                                return Some(*x as char);
                            }

                            None
                        })
                        .collect::<String>()
                        .trim()
                        .to_string();

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

                Err(_) => ()
            }
        });

        Client {
            stream: stream_clone,
            socket_addr,
            max_buffer,
            thread: Some(thread),
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        match self.stream.shutdown(Shutdown::Both) {
            Ok(_) => (),
            Err(ref e) if e.kind() == std::io::ErrorKind::NotConnected => (),
            Err(_) => (),
        }

        if let Some(thread) = self.thread.take() {
            thread.join().expect(&format!(
                "Trying to terminate thread of client {} but failed.",
                self.socket_addr
            ));
        }

        println!("Connection with {} successfully closed!", self.socket_addr);
    }
}
