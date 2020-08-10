mod server;
mod client;

use server::Server;

fn main() {
    let _server = Server::new("0.0.0.0:2424")
        .max_clients(4)
        .run();
}