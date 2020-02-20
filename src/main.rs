#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod server;
use server::GdbServer;

fn main() {
    println!("started program");
    GdbServer::start_server();
}
