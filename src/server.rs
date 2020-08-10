use std::net::{IpAddr, Ipv4Addr};
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::net::TcpListener;
use std::net::Shutdown;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::client::Client;
use super::client::ClientPayload;
use super::client::PayloadSignal;


pub struct Server {
    server_address: SocketAddr,
    max_clients: usize,

    clients: Vec<Client>,
    has_running: bool,
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
            max_clients: 4,
            clients: Vec::with_capacity(4),
            has_running: false,
        }
    }

    pub fn max_clients(mut self, max_clients: usize) -> Server {
        if max_clients == self.max_clients {
            return self
        }

        self.max_clients = max_clients;
        self.clients = Vec::with_capacity(max_clients);

        self
    }

    pub fn run(mut self) -> Option<Server> {
        if self.has_running {
            return None
        }

        let listener = TcpListener::bind(self.server_address).expect("Could not run the server, maybe the address and port already reserved?");
        listener.set_nonblocking(true).expect("Could not run the server as non-blocking");

        let (tx, rx) = mpsc::channel::<ClientPayload>();
        
        loop {
            if !self.has_running {
                self.has_running = true;
            }

            if let Ok((stream, socket_addr)) = listener.accept() {
                /* Creating new session */
                let client_id = self.clients.len();
                let sender = tx.clone();
                let client = Client::new(client_id, stream, socket_addr, sender);
                self.clients.push(client);
            }

            if let Ok((client_id, socket_addr, message, payload_signal)) = rx.try_recv() {
                match payload_signal {
                    PayloadSignal::InterruptSignal => {
                        // Clients need to be stopped
                        if let Some(client) = self.clients.get_mut(client_id) {
                            client.stream.shutdown(Shutdown::Both).ok();
                        }

                        /*
                            NOTE|CRITICAL:

                            It will throw panic when the index > len.

                            How to reproduce:
                            Clients connected with id 0, 1 respectively, and then
                            client 0 disconnect first. After 1 going to disconnect
                            the panic occur.

                            Workaround: Use better approach like use `.iter` to validate id or
                            use map for storing clients.
                        */
                        self.clients.remove(client_id);
                    }

                    _ => {
                        println!("{}@{} -> {}", client_id, socket_addr, message);
                    }
                }
            }
            
            thread::sleep(Duration::from_millis(1));
        }
    }
}