/* This example demonstrates how to subscribe to a topic.
*/

use bogobarf::create_client;

fn main() {
    println!("Listener is starting");

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let client_node = create_client().await.unwrap();
        client_node.bye().await.unwrap();
    });
}
