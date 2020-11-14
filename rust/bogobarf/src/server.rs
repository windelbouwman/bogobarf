use std::collections::HashMap;

extern crate tokio;
use crate::fancyheader::print_header;
use crate::peer::process_client;
use tokio::net::TcpListener;

pub fn start_server() {
    let server = Server::default();
    server.run();
}

#[derive(Default, Debug)]
struct Peer {}

#[derive(Default, Debug)]
struct Server {
    pub state: ServerState,
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
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            match self.run_async().await {
                Ok(()) => {
                    info!("Server is done");
                }
                Err(err) => {
                    error!("Error in server: {:?}", err);
                }
            }
        });
    }

    async fn run_async(&self) -> Result<(), std::io::Error> {
        let addr: std::net::SocketAddr = "127.0.0.1:6142".parse().unwrap();

        info!("Server listening on {:?}", addr);
        let listener = TcpListener::bind(&addr).await?;

        loop {
            let (socket, _address) = listener.accept().await?;
            process_client(socket);
        }
    }
}
