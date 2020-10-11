extern crate tokio;
use tokio::net::TcpStream;

use crate::connection::Connection;
use crate::message::Message;
use futures::future::Future;
use futures::oneshot;
use futures::sync::{mpsc, oneshot::Sender};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::prelude::*;

#[derive(Debug)]
pub struct ClientNode {
    seq_nr: Mutex<RefCell<u32>>,
    connection: Connection,
    subscriptions: Mutex<RefCell<HashMap<String, mpsc::UnboundedSender<String>>>>,
    client_data: Arc<ClientData>,
}

#[derive(Debug)]
pub struct ClientData {
    response_queues: Mutex<RefCell<HashMap<u32, Sender<Message>>>>,
}

impl ClientNode {
    fn new(stream: TcpStream) -> Self {
        ClientNode {
            seq_nr: Mutex::new(RefCell::new(42)),
            connection: Connection::new(stream),
            subscriptions: Default::default(),
            client_data: Arc::new(ClientData {
                response_queues: Default::default(),
            }),
        }
    }

    fn get_seq_nr(&self) -> u32 {
        let seq_refcell = self.seq_nr.lock().unwrap();
        let seq_nr: u32 = *seq_refcell.borrow();
        seq_refcell.replace(seq_nr + 1);
        seq_nr
    }

    fn register(&self, name: String) -> impl Future<Item = (), Error = ()> {
        self.call_method("register".to_string(), vec![name])
            .map(|_| ())
    }

    /// Return a stream of topic publications!
    pub fn subscribe(&self, topic: String) -> impl Stream<Item = String, Error = ()> {
        let (tx, rx) = mpsc::unbounded::<String>();
        self.subscriptions
            .lock()
            .unwrap()
            .borrow_mut()
            .insert(topic.clone(), tx);
        self.call_method("subscribe".to_string(), vec![topic]);
        rx
    }

    /// Get the list of topics on the system.
    pub fn topic_list(&self) -> impl Future<Item = Vec<String>, Error = ()> {
        self.call_method("topic_list".to_string(), vec![])
            .map(|_| vec!["a".to_string()])
    }

    pub fn publish(&self, topic: String, value: String) -> impl Future<Item = (), Error = ()> {
        let req_message = Message::Publish { topic, value };
        self.send_message(req_message)
    }

    fn send_message(&self, message: Message) -> impl Future<Item = (), Error = ()> {
        self.connection
            .send_message(message)
            .map(|_| debug!("Send complete"))
            .map_err(|e| {
                println!("Error! {}", e);
            })
    }

    pub fn bye(&self) -> impl Future<Item = (), Error = ()> {
        let message = Message::Bye;
        self.send_message(message)
    }

    fn call_method(
        &self,
        method_name: String,
        args: Vec<String>,
    ) -> impl Future<Item = String, Error = ()> {
        debug!("Invoking {}", method_name);
        let seq_nr = self.get_seq_nr();

        let (tx, rx) = oneshot::<Message>();

        // Create response object:
        self.client_data
            .response_queues
            .lock()
            .unwrap()
            .borrow_mut()
            .insert(seq_nr, tx);

        // Now send request.
        let req_message = Message::RpcRequest {
            sequence_id: seq_nr,
            method: method_name,
            args,
        };

        self.send_message(req_message).and_then(move |()| {
            // Wait for response here.
            rx.into_future()
                .and_then(|message| {
                    let result = if let Message::RpcResponse { result, .. } = message {
                        result
                    } else {
                        panic!("Response must be rpc response!");
                    };
                    Ok(result)
                })
                .map_err(|_| {
                    error!("Error in split!");
                })
        })
    }

    /*
    fn handle_message(&self, message: Message) -> impl Future<Item=(), Error=()> {
        debug!("handling message {:?}", message);
        match message {
            Message::Publish { topic, value } => {
                if let Some(handle_queue) = self.subscriptions.lock().unwrap().borrow().get(&topic) {
                    handle_queue.clone().send(value)
                    .map(|_| Ok(()))
                    .into_future()
                } else {
                    futures::finished(Ok(()))
                    .into_future()
                }
            },
            x => {
                error!("Unhandled message {:?}", x);
            }
        }
    }
    */
}

pub fn create_client() -> impl Future<Item = ClientNode, Error = ()> {
    let addr = "127.0.0.1:6142".parse().unwrap();
    TcpStream::connect(&addr)
        .map_err(|err| {
            error!("Got error connecting: {:?}", err);
        })
        .and_then(|stream| {
            debug!("Connected!");
            let client_node = ClientNode::new(stream);
            client_node.register("rust-client".to_string()).map(|_| {
                debug!("Call completed!");
                client_node
            })
        })
}
