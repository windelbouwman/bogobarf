extern crate tokio;
use crate::message::Message;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

pub fn process_client(socket: TcpStream) {
    info!("Got incoming socket! {:?}", socket);
    let (read_half, write_half) = socket.into_split();
    let mut framed_stream = FramedRead::new(read_half, LengthDelimitedCodec::new());
    let mut framed_sink = FramedWrite::new(write_half, LengthDelimitedCodec::new());

    let (tx, mut rx) = mpsc::channel::<Message>(25);

    // tokio::spawn(thingy2);
    tokio::spawn(async move {
        while let Some(msg) = rx.next().await {
            info!("Tx-ing {:?}", msg);
            let data = msg.to_bytes();
            framed_sink.send(data).await.unwrap();
        }
    });

    // Start message processor:

    tokio::spawn(async move {
        while let Some(packet) = framed_stream.next().await {
            // debug!("Got: {:?}", &packet);
            // try to decode cbor package:
            let p2: Vec<u8> = packet.unwrap().to_vec();
            let message: Message = serde_cbor::from_slice(&p2).unwrap();

            info!("Received message: {:?}", message);

            // Create response and place onto queue!
            let message2 = Message::RpcResponse {
                sequence_id: 1337,
                result: "fubar".to_string(),
            };

            tx.send(message2).await.unwrap();
            // Ok(())
            // })
            // .map_err(|err| println!("Failed: {:?}", err));
        }
    });
}
