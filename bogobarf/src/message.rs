
use serde::{Serialize, Deserialize};
// use tokio::futures::Sink;
// use serde_cbor;
// use bytes::Bytes;
// use tokio::codec::{Framed, LengthDelimitedCodec};
// use std::iter::FromIterator;
// use futures::Sink;

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub id: i64,
    pub text: String,
}

pub struct RpcRequest {
    pub sequence_id: i64,
    pub method: String,
}

pub struct RpcResponse {
    pub sequence_id: i64,
}

// Wire message:
pub enum Message2 {
    RpcRequest(RpcRequest),
    RpcResponse(RpcResponse),
    Event,
}

/*
pub fn tx(message: Message, sink: Sink<LengthDelimitedCodec>) -> Send {
    let bytes = serde_cbor::to_vec(&message).unwrap();
    let data = Bytes::from_iter(bytes.iter());
    sink.send(data)
}
*/
