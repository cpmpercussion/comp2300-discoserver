#![allow(dead_code)]

mod server;
use server::GdbServer;

fn main() {
    if get_version_from_argv() {
        println!("disco-emulator v{}.{}.{}", 1, 1, 0);
        return;
    }

    println!("started emulator server");
    GdbServer::start_server();
}

fn get_version_from_argv() -> bool {
    let mut args = std::env::args();
    while let Some(arg) = args.next() {
        if arg == "-v" || arg == "--version" {
            return true;
        }
    }
    return false;
}
