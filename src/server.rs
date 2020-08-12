use std::collections::HashMap;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr};
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::client::Client;
use super::client::ClientPayload;
use super::client::PayloadSignal;


pub struct Server {
    server_address: SocketAddr,
    max_clients: usize,
    clients: HashMap<SocketAddr, Client>,
}

impl Server {
    pub fn new<A: ToSocketAddrs>(a: A) -> Server {
        let server_address = match a.to_socket_addrs() {
            Ok(mut addrs) => {
                if let Some(addr) = addrs.next() {
                    addr
                } else {
                    println!("Could not run the server on that address, use fallback instead: 127.0.0.1:2424");

                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 2424)
                }
            }

            Err(_) => panic!("Invalid address given"),
        };

        Server {
            server_address,
            max_clients: 1,
            clients: HashMap::with_capacity(1),
        }
    }

    pub fn max_clients(mut self, max_clients: usize) -> Server {
        if max_clients == self.max_clients {
            return self
        }

        self.max_clients = max_clients;
        self.clients = HashMap::with_capacity(max_clients);

        self
    }

    pub fn run(mut self) {
        let listener = TcpListener::bind(self.server_address).expect("Could not run the server, maybe the address and port already reserved?");
        listener.set_nonblocking(true).expect("Could not run the server as non-blocking");

        println!("Server running on tcp://{}", self.server_address);
        println!("Setting maximuum client to {}", self.max_clients);

        let (tx, rx) = mpsc::channel::<ClientPayload>();

        loop {
            if let Ok((stream, socket_addr)) = listener.accept() {
                /* Dropping new incoming socket if the server full already */
                if self.clients.len() == self.max_clients && !self.clients.contains_key(&socket_addr) {
                    continue;
                }

                /* Creating new session */
                let sender = tx.clone();
                let client = Client::new(stream, socket_addr, sender);

                self.clients.insert(socket_addr, client);
            }

            if let Ok((socket_addr, payload_signal, message)) = rx.try_recv() {
                match payload_signal {
                    PayloadSignal::InterruptSignal => {
                        self.clients.remove(&socket_addr);
                    }

                    _ => {
                        if let Some(message) = message {
                            let fmt = format!("{} -> {}\r\n", socket_addr, message);
                            print!("{}", fmt);

                            self.clients = self.clients.into_iter().filter_map(|(k, mut v)| {
                                if socket_addr != k {
                                    v.stream.write(fmt.as_bytes()).ok();
                                }

                                Some((k, v))
                            }).collect();
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(1));
        }
    }
}