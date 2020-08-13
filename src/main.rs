mod client;
mod config;
mod server;

use config::Config;
use server::Server;

fn main() {
    let config = Config::init();

    Server::new(&config.get_address())
        .max_clients(config.get_max_clients())
        .max_acceptable_buffer(config.get_acceptable_buffer())
        .run();
}
