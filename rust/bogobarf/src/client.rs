extern crate tokio;
use tokio::net::TcpStream;

use crate::connection::Connection;
use crate::message::Message;
use futures::StreamExt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub struct ClientNode {
    seq_nr: Mutex<RefCell<u32>>,
    connection: Connection,
    client_data: Arc<ClientData>,
}

#[derive(Debug)]
pub struct ClientData {
    response_queues: Mutex<RefCell<HashMap<u32, oneshot::Sender<Message>>>>,
    subscriptions: Mutex<RefCell<HashMap<String, mpsc::Sender<String>>>>,
}

#[derive(Debug)]
pub enum BogoBarfError {
    // Other,
    Io(std::io::Error),
}

impl ClientNode {
    fn new(stream: TcpStream) -> (Self, mpsc::Receiver<Message>) {
        let (connection, rx2) = Connection::new(stream);
        let client_data = Arc::new(ClientData {
            response_queues: Default::default(),
            subscriptions: Default::default(),
        });

        (
            ClientNode {
                seq_nr: Mutex::new(RefCell::new(42)),
                connection,
                client_data,
            },
            rx2,
        )
    }

    fn get_seq_nr(&self) -> u32 {
        let seq_refcell = self.seq_nr.lock().unwrap();
        let seq_nr: u32 = *seq_refcell.borrow();
        seq_refcell.replace(seq_nr + 1);
        seq_nr
    }

    async fn register(&self, name: String) -> Result<(), BogoBarfError> {
        let res = self.call_method("register".to_string(), vec![name]).await?;
        info!("register result = {}", res);
        Ok(())
    }

    /// Return a stream of topic publications!
    pub async fn subscribe(&self, topic: String) -> Result<mpsc::Receiver<String>, BogoBarfError> {
        let (tx, rx) = mpsc::channel::<String>(27);
        self.client_data
            .subscriptions
            .lock()
            .unwrap()
            .borrow_mut()
            .insert(topic.clone(), tx);
        let res = self
            .call_method("subscribe".to_string(), vec![topic])
            .await?;
        info!("subscribe result = {}", res);
        Ok(rx)
    }

    /// Get the list of topics on the system.
    pub async fn topic_list(&self) -> Result<Vec<String>, BogoBarfError> {
        let res = self.call_method("topic_list".to_string(), vec![]).await?;
        Ok(vec![res])
    }

    pub async fn publish(&self, topic: String, value: String) -> Result<(), BogoBarfError> {
        let req_message = Message::Publish { topic, value };
        self.send_message(req_message).await?;
        Ok(())
    }

    async fn send_message(&self, message: Message) -> Result<(), BogoBarfError> {
        self.connection.send_message(message).await;
        debug!("Send complete");
        Ok(())
    }

    pub async fn bye(&self) -> Result<(), BogoBarfError> {
        let message = Message::Bye;
        self.send_message(message).await?;
        Ok(())
    }

    async fn call_method(
        &self,
        method_name: String,
        args: Vec<String>,
    ) -> Result<String, BogoBarfError> {
        debug!("Invoking {}", method_name);
        let seq_nr = self.get_seq_nr();

        let (tx, rx) = oneshot::channel::<Message>();

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

        self.send_message(req_message).await?;

        // Wait for response here.
        let message: Message = rx.await.unwrap();
        // .map_err(|_| {
        //     error!("Error in split!");
        // })

        let result = if let Message::RpcResponse { result, .. } = message {
            result
        } else {
            panic!("Response must be rpc response!");
        };
        Ok(result)
    }
}

async fn message_processing(client_data: Arc<ClientData>, mut rx2: mpsc::Receiver<Message>) {
    while let Some(msg) = rx2.next().await {
        handle_message(&client_data, msg).await;
    }
}

async fn handle_message(client_data: &Arc<ClientData>, message: Message) {
    debug!("handling message {:?}", message);
    match message {
        Message::Publish { topic, value } => {
            let q2 = if let Some(handle_queue) = client_data
                .subscriptions
                .lock()
                .unwrap()
                .borrow()
                .get(&topic)
            {
                Some(handle_queue.clone())
            } else {
                error!("No more subbed to this topic: {}", topic);
                None
            };

            if let Some(q3) = q2 {
                q3.send(value).await.unwrap();
            }
        }
        Message::RpcResponse {
            sequence_id,
            result,
        } => {
            let slot = client_data
                .response_queues
                .lock()
                .unwrap()
                .borrow_mut()
                .remove(&sequence_id);

            if let Some(slot) = slot {
                slot.send(Message::RpcResponse {
                    sequence_id,
                    result,
                })
                .unwrap();
            } else {
                error!("Spurious response!");
            }
        }
        x => {
            error!("Unhandled message {:?}", x);
        }
    }
}

impl From<std::io::Error> for BogoBarfError {
    fn from(err: std::io::Error) -> Self {
        BogoBarfError::Io(err)
    }
}

pub async fn create_client() -> Result<ClientNode, BogoBarfError> {
    let addr: std::net::SocketAddr = "127.0.0.1:6142".parse().unwrap();
    let stream = TcpStream::connect(&addr).await?;

    debug!("Connected!");
    let (client_node, rx2) = ClientNode::new(stream);
    let client_data2 = client_node.client_data.clone();

    // Spawn message processing thread:
    tokio::spawn(async {
        message_processing(client_data2, rx2).await;
    });
    client_node.register("rust-client".to_string()).await?;

    debug!("Call completed!");
    Ok(client_node)
}
