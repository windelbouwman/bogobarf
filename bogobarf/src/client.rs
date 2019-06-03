

extern crate tokio;
// use tokio::io;
use tokio::net::{TcpStream};
use tokio::prelude::*;
use tokio::codec::LengthDelimitedCodec;
// use bytes::bytes::By
extern crate bytes;
use bytes::Bytes;
use crate::message::{Message};
// use serde;
use std::iter::FromIterator;
use serde_cbor;

pub fn ping() {
    info!("PeNGG");

    let addr = "127.0.0.1:6142".parse().unwrap();
    let client = TcpStream::connect(&addr).and_then(|stream| {
        
        let framed_stream = stream.framed(LengthDelimitedCodec::new());
        info!("Connected!");
        let message = Message { id: 42, text: "barf bogobarf\n".to_string() };
        //Bytes::from_static("barf bogobarf\n".as_bytes())
        let bytes = serde_cbor::to_vec(&message).unwrap();
        let data = Bytes::from_iter(bytes.iter());
        // tx(message, framed_stream)
        framed_stream.send(data)
        .and_then(|framed_stream| {
            debug!("Wrote to stream!");
            let message = Message { id: 1337, text: "barf bogobarf222\n".to_string() };
            // tx(message, framed_stream)
            let bytes = serde_cbor::to_vec(&message).unwrap();
            let data = Bytes::from_iter(bytes.iter());
            framed_stream.send(data)
        })
        .and_then(|_| {
            debug!("Wrote to stream again!");
            Ok(())
        })
    })
    .map_err(|err| {
        error!("Got error connecting: {:?}", err);
    });
    tokio::run(client);
}
