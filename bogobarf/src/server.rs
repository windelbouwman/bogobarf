
use std::collections::HashMap;

extern crate tokio;
use crate::fancyheader::print_header;
use crate::peer::process_client;
use tokio::net::{TcpListener};
use tokio::prelude::*;

pub fn start_server() {
    let server = Server::default();
    server.run();
}

#[derive(Default, Debug)]
struct Peer {

}

#[derive(Default, Debug)]
struct Server {
    pub state: ServerState
}

#[derive(Default, Debug)]
struct ServerState {
    pub clients: HashMap<String, Peer>,
    pub topics: HashMap<String, String>,
}

impl Server {
    fn run(&self) {
        print_header();

        info!("Starting server");
        let addr = "127.0.0.1:6142".parse().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();

        let server = listener.incoming().for_each(|socket| {
            process_client(socket);
            Ok(())
        })
        .map_err(|err| {
            println!("Error in accept: {:?}", err);
        });

        info!("Server listening on {:?}", addr);
        tokio::run(server);
    }
}
