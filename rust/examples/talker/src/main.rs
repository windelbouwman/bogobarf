/*
 This example demonstrates how to publish a value in bogobarf.
*/

use bogobarf::create_client;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        println!("Talker starts talking.");

        let client_node = create_client().await.unwrap();
        client_node
            .publish("/chatter".to_string(), "Hello world".to_string())
            .await
            .unwrap();
        client_node.bye().await.unwrap();
    });
}
