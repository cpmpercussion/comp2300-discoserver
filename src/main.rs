#![allow(dead_code)]

mod server;
use server::GdbServer;

fn main() {
    println!("started program");
    GdbServer::start_server();
}
