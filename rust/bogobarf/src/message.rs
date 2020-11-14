use serde::{Deserialize, Serialize};
// use serde_cbor;
use bytes::Bytes;
// use tokio::codec::{Framed, LengthDelimitedCodec};
use std::iter::FromIterator;
// use futures::Sink;

#[derive(Serialize, Deserialize, Debug)]
pub struct OldMessage {
    pub id: i64,
    pub text: String,
}

// Wire message:
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "call")]
    RpcRequest {
        #[serde(rename = "seq")]
        sequence_id: u32,
        method: String,
        args: Vec<String>,
    },

    #[serde(rename = "ret")]
    RpcResponse {
        #[serde(rename = "seq")]
        sequence_id: u32,

        result: String,
    },

    #[serde(rename = "pub")]
    Publish {
        topic: String,
        value: String,
    },

    #[serde(rename = "bye")]
    Bye,

    Event,
}

impl Message {
    pub fn to_bytes(&self) -> Bytes {
        let bytes: Vec<u8> = serde_cbor::to_vec(&self).unwrap();
        Bytes::from_iter(bytes.into_iter())
    }
}

/*
pub fn tx(message: Message, sink: Sink<LengthDelimitedCodec>) -> Send {
    let bytes = serde_cbor::to_vec(&message).unwrap();
    let data = Bytes::from_iter(bytes.iter());
    sink.send(data)
}
*/
