mod client;
mod config;
mod server;

use server::Server;
use config::Config;

fn main() {
    let config = Config::init();

    Server::new(&config.get_address())
        .max_clients(config.get_max_clients())
        .max_acceptable_buffer(config.get_acceptable_buffer())
        .run();
}