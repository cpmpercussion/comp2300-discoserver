#![allow(dead_code)]

mod server;
use server::GdbServer;

fn main() {
    println!("started emulator server");
    GdbServer::start_server();
}
