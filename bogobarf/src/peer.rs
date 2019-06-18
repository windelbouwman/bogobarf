

extern crate tokio;
use tokio::net::{TcpStream};
use tokio::prelude::*;
use tokio::io;
use tokio::codec::{LengthDelimitedCodec, Framed};
use crate::message::OldMessage;
use futures::sync::mpsc;
use bytes::Bytes;
use std::iter::FromIterator;
use futures::stream::SplitSink;

pub fn process_client(socket: TcpStream) {
    info!("Got incoming socket! {:?}", socket);
    // socket
    // let (framed_sink, framed_stream)
    let framed_socket = Framed::new(socket, LengthDelimitedCodec::new());

    let (framed_sink, framed_stream) = framed_socket.split();
    // tx_task(framed_sink);

    let (tx, rx) = mpsc::unbounded::<OldMessage>();

    // fn do_tx(msg: Message) -> IntoFuture<Item=(), Error=()> {
    //     let bytes = serde_cbor::to_vec(&msg).unwrap();
    //     let data = Bytes::from_iter(bytes.iter());
    //     Ok(())
    // }

    // Start message processor:
    let thingy2 = rx.fold(framed_sink, |framed_sink2, msg: OldMessage| {
        info!("Tx-ing {:?}", msg);
        let bytes = serde_cbor::to_vec(&msg).unwrap();
        let data = Bytes::from_iter(bytes.iter());
        framed_sink2.send(data)
        // .map(|_| ())
        .map_err(|err| println!("Failed: {:?}", err))
    })
    .map(|_| ())
    .map_err(|err| println!("Failed: {:?}", err));
    tokio::spawn(thingy2);

    let thingy = framed_stream.for_each(move |packet| {
        // debug!("Got: {:?}", &packet);
        // try to decode cbor package:
        let message: OldMessage = serde_cbor::from_slice(&packet).unwrap();

        info!("Received message: {:?}", message);

        // Create response and place onto queue!
        let message2 = OldMessage { id: 1337, text: "Response!".to_string() };

        tx.clone().send(message2)
        .map(|_| ())
        .map_err(|_| io::ErrorKind::Other.into())
        // Ok(())
    })
    .map_err(|err| println!("Failed: {:?}", err));

    tokio::spawn(thingy);
}

fn recv() {
    
}
