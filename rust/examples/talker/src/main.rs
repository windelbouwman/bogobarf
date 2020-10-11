/*
 This example demonstrates how to publish a value in bogobarf.
*/

use bogobarf::create_client;
use tokio::prelude::*;

fn main() {
    println!("Talker starts talking.");

    let task = create_client().and_then(|client_node| {
        client_node
            .publish("/chatter".to_string(), "Hello world".to_string())
            .and_then(move |_| client_node.bye())
    });
    tokio::run(task);
}
