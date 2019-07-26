/* This example demonstrates how to subscribe to a topic.
*/

use bogobarf::create_client;
use tokio::prelude::*;

fn main() {
    println!("Listener is starting");
    let task = create_client().and_then(|client_node| {
        client_node
            .publish("/chatter".to_string(), "Hello world".to_string())
            .and_then(move |_| client_node.bye())
    });
    tokio::run(task);
}
