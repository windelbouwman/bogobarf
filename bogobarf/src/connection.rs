// Abstraction which can be used on client and on server
// Create an object to which messages can be send, and
// also messages can be received.

use tokio::codec::{Framed, LengthDelimitedCodec};
use tokio::net::TcpStream;
use tokio::prelude::*;

extern crate bytes;

// use serde;
use futures::sync::mpsc;
use serde_cbor;
// use std::sync::mpsc::{channel, Sender};

use crate::message::Message;

pub struct Connection {
    tx: mpsc::UnboundedSender<Message>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        let (framed_sink, framed_stream) = Framed::new(stream, LengthDelimitedCodec::new()).split();
        let (tx, rx) = mpsc::unbounded();

        // Start tx thread
        let tx_thread = rx
            .fold(framed_sink, |framed_sink2, message: Message| {
                let data = message.to_bytes();
                info!("Sending out (in middle of fold!) {:?}", data);
                framed_sink2.send(data).map_err(|_| ())
            })
            .map(|_| ());
        tokio::spawn(tx_thread);

        // Start rx thread:
        let rx_thread = framed_stream
            .for_each(|packet| {
                info!("Incoming data: {:?}", packet);
                let message: Message = serde_cbor::from_slice(&packet).unwrap();
                info!("Received message: {:?}", message);
                Ok(())
            })
            .map_err(|e| println!("Error! {:?}", e));
        tokio::spawn(rx_thread);

        Connection { tx }
    }

    pub fn send_message(
        &self,
        message: Message,
    ) -> futures::sink::Send<mpsc::UnboundedSender<Message>> {
        info!("Tx message: {:?}", message);
        self.tx.clone().send(message)
    }

    /*
        fn recv_msg(&self) -> Stream<Message> {
        }
    */
}
