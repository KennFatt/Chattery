mod server;
mod client;

use server::Server;

fn main() {
    Server::new("0.0.0.0:2424")
        .max_clients(1)
        .run();
}