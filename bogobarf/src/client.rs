

extern crate tokio;
// use tokio::io;
use tokio::net::{TcpStream};
use tokio::prelude::*;
// use bytes::bytes::By

use crate::message::Message;
use crate::connection::Connection;
use futures::future::Future;

struct ClientNode {
    seq_nr: u32,
    connection: Connection,
}

impl ClientNode {
    fn new(stream: TcpStream) -> Self {
        ClientNode {
            seq_nr: 42,
            connection: Connection::new(stream),
        }
    }

    fn get_seq_nr(&mut self) -> u32 {
        let seq_nr = self.seq_nr;
        self.seq_nr = seq_nr + 1;
        seq_nr
    }

    fn subscribe(&mut self, topic: String) {
        self.call_method("sub".to_string(), vec![topic]);
    }

    fn publish(&mut self, topic: String, value: String) {
        let req_message = Message::Publish {
            topic, value
        };
        let task = self.connection.send_message(req_message)
        .map(|_| { debug!("Send complete")})
        .map_err(|e| { println!("Error! {}", e); });
        tokio::spawn(task);
    }

    fn call_method(&mut self, method_name: String, args: Vec<String>) { // -> dyn Future<Item=(),Error=()> {
        info!("Invoking {}", method_name);
        let seq_nr = self.get_seq_nr();
        let req_message = Message::RpcRequest { 
            sequence_id: seq_nr, method: method_name, 
            args};
        let task = self.connection.send_message(req_message)
        .map(|_| { debug!("Send complete")})
        .map_err(|e| { println!("Error! {}", e); });
        tokio::spawn(task);
        // .map(|x| ())
/*
        // await recv_queue()
    */
    }
}

pub fn ping() {
    info!("PeNGG");

    let addr = "127.0.0.1:6142".parse().unwrap();
    let client = TcpStream::connect(&addr)
    .map_err(|err| {
        error!("Got error connecting: {:?}", err);
    })
    .and_then(|stream| {
        info!("Connected!");
        let mut client_node = ClientNode::new(stream);
        client_node.call_method("register".to_string(), vec!["rust-client".to_string()]);
        client_node.publish("fuu".to_string(), "bar".to_string());
        // let message = OldMessage { id: 42, text: "barf bogobarf\n".to_string() };
        //Bytes::from_static("barf bogobarf\n".as_bytes())
        // let bytes = serde_cbor::to_vec(&message).unwrap();
        // let data = Bytes::from_iter(bytes.iter());
        // tx(message, framed_stream)
        // framed_stream.send(data)
        // .and_then(|framed_stream| {
        //     debug!("Wrote to stream!");
        //     let message = OldMessage { id: 1337, text: "barf bogobarf222\n".to_string() };
        //     // tx(message, framed_stream)
        //     let bytes = serde_cbor::to_vec(&message).unwrap();
        //     let data = Bytes::from_iter(bytes.iter());
        //     framed_stream.send(data)
        // })
        // .and_then(|_| {
        //     debug!("Wrote to stream again!");
        //     Ok(())
        // })
        use tokio::timer::Delay;
        use std::time::{Duration, Instant};
        let when = Instant::now() + Duration::from_millis(200);
        Delay::new(when)
        .and_then(|_| {
            info!("Timer expired!");
            Ok(())
        })
        .map_err(|e| {
            println!("Error: {}", e);
            // Ok(())
        })
    });
    tokio::run(client);
}
