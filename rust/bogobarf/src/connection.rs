// Abstraction which can be used on client and on server
// Create an object to which messages can be send, and
// also messages can be received.

use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

extern crate bytes;

// use serde;
// use futures::sync::mpsc;
use serde_cbor;
// use std::sync::mpsc::{channel, Sender};

use crate::message::Message;

#[derive(Debug)]
pub struct Connection {
    tx: mpsc::Sender<Message>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> (Self, mpsc::Receiver<Message>) {
        let (read_half, write_half) = stream.into_split();

        // let (framed_sink, framed_stream) = Framed::new(stream, LengthDelimitedCodec::new()).split();
        let mut framed_write = FramedWrite::new(write_half, LengthDelimitedCodec::new());
        let (tx, mut rx) = mpsc::channel::<Message>(25);

        // Start tx thread
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                let data = message.to_bytes();
                debug!("Sending out (in middle of fold!) {:?}", data);
                framed_write.send(data).await.unwrap();
            }
        });

        let mut framed_read = FramedRead::new(read_half, LengthDelimitedCodec::new());
        // Start rx thread:
        // let (tx2, _rx2) = mpsc::unbounded();
        let (tx2, rx2) = mpsc::channel::<Message>(25);
        tokio::spawn(async move {
            while let Some(packet) = framed_read.next().await {
                debug!("Incoming data: {:?}", packet);
                let p2: Vec<u8> = packet.unwrap().to_vec();
                match serde_cbor::from_slice::<Message>(&p2) {
                    Ok(message) => {
                        debug!("Received message: {:?}", message);
                        tx2.send(message).await.unwrap();
                    }
                    Err(e) => {
                        error!("Error: {:?}", e);
                    }
                }
            }
        });

        (Connection { tx }, rx2)
    }

    pub async fn send_message(&self, message: Message) {
        debug!("Tx message: {:?}", message);
        self.tx.clone().send(message).await.unwrap();
    }

    /*
        fn recv_msg(&self) -> Stream<Message> {
        }
    */
}
