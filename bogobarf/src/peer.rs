extern crate tokio;
use crate::message::Message;
use futures::sync::mpsc;
use tokio::codec::{Framed, LengthDelimitedCodec};
use tokio::io;
use tokio::net::TcpStream;
use tokio::prelude::*;

pub fn process_client(socket: TcpStream) {
    info!("Got incoming socket! {:?}", socket);
    let (framed_sink, framed_stream) = Framed::new(socket, LengthDelimitedCodec::new()).split();

    let (tx, rx) = mpsc::unbounded::<Message>();

    // Start message processor:
    let thingy2 = rx
        .fold(framed_sink, |framed_sink2, msg| {
            info!("Tx-ing {:?}", msg);
            let data = msg.to_bytes();
            framed_sink2
                .send(data)
                // .map(|_| ())
                .map_err(|err| println!("Failed: {:?}", err))
        })
        .map(|_| ())
        .map_err(|err| println!("Failed: {:?}", err));
    tokio::spawn(thingy2);

    let thingy = framed_stream
        .for_each(move |packet| {
            // debug!("Got: {:?}", &packet);
            // try to decode cbor package:
            let message: Message = serde_cbor::from_slice(&packet).unwrap();

            info!("Received message: {:?}", message);

            // Create response and place onto queue!
            let message2 = Message::RpcResponse { sequence_id: 1337 };

            tx.clone()
                .send(message2)
                .map(|_| ())
                .map_err(|_| io::ErrorKind::Other.into())
            // Ok(())
        })
        .map_err(|err| println!("Failed: {:?}", err));

    tokio::spawn(thingy);
}
