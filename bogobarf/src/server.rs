
use std::collections::HashMap;

extern crate tokio;
use crate::fancyheader::print_header;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::io;
use tokio::codec::{LengthDelimitedCodec, Framed};
use crate::message::Message;
use futures::sync::mpsc;
use bytes::Bytes;
use std::iter::FromIterator;

pub fn start_server() {
    let server = Server::default();
    server.run();
}

#[derive(Default, Debug)]
struct Node {

}

#[derive(Default, Debug)]
struct Server {
    pub state: ServerState
}

#[derive(Default, Debug)]
struct ServerState {
    pub clients: HashMap<String, Node>,
    pub topics: HashMap<String, String>,
}

impl Server {
    fn run(&self) {
        print_header();

        info!("Starting server");
        let addr = "127.0.0.1:6142".parse().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();

        let server = listener.incoming().for_each(move |socket| {
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

fn process_client(socket: TcpStream) {
    info!("Got incoming socket! {:?}", socket);
    // socket
    let framed_socket = Framed::new(socket, LengthDelimitedCodec::new());

    let (framed_sink, framed_stream) = framed_socket.split();

    let (tx, rx) = mpsc::unbounded::<Message>();

    // fn do_tx(msg: Message) -> IntoFuture<Item=(), Error=()> {
    //     let bytes = serde_cbor::to_vec(&msg).unwrap();
    //     let data = Bytes::from_iter(bytes.iter());
    //     Ok(())
    // }

    // Start message processor:
    // let thingy2 = rx.fold(framed_sink, |framed_sink2, msg: Message| {

    let thingy2 = rx.for_each(|msg| {
        info!("Tx-ing {:?}", msg);
        
        let bytes = serde_cbor::to_vec(&msg).unwrap();
        let data = Bytes::from_iter(bytes.iter());
        // framed_sink2.send(data)
        // .map(|_| ())
        // .map_err(|err| println!("Failed: {:?}", err));

        // Ok(framed_sink2)
        Ok(())
    })
    .map(|_| ())
    .map_err(|err| println!("Failed: {:?}", err));
    tokio::spawn(thingy2);

    let thingy = framed_stream.for_each(move |packet| {
        // debug!("Got: {:?}", &packet);
        // try to decode cbor package:
        let message: Message = serde_cbor::from_slice(&packet).unwrap();

        info!("Received message: {:?}", message);

        // Create response and place onto queue!
        let message2 = Message { id: 1337, text: "Response!".to_string() };

        tx.clone().send(message2)
        .map(|_| ())
        .map_err(|_| io::ErrorKind::Other.into())
        // Ok(())
    })
    .map_err(|err| println!("Failed: {:?}", err));

    tokio::spawn(thingy);
}
