#[macro_use]
extern crate log;
extern crate simplelog;
use simplelog::{Config, LevelFilter, TermLogger};
extern crate clap;
use clap::{App, Arg, SubCommand};

extern crate tokio;

// crate bogobarf;
use bogobarf::create_client;
use bogobarf::server;

fn main() {
    let matches = App::new("bogobarf")
        .version("0.0.2")
        .about("Robotic middleware")
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(SubCommand::with_name("serve").about("Starts the server"))
        .subcommand(SubCommand::with_name("ping").about("Pings the server"))
        .subcommand(SubCommand::with_name("list").about("List topics on the server"))
        .subcommand(
            SubCommand::with_name("echo").about("Echo a topic").arg(
                Arg::with_name("topic")
                    .required(true)
                    .help("The topic to publish to"),
            ),
        )
        .subcommand(
            SubCommand::with_name("pub")
                .about("Publish data onto a topic")
                .arg(
                    Arg::with_name("topic")
                        .required(true)
                        .help("The topic to publish to"),
                )
                .arg(Arg::with_name("value").required(true).help("The value")),
        )
        .get_matches();

    // Setup logger:
    let level_filter = match matches.occurrences_of("v") {
        1 => LevelFilter::Debug,
        _ => LevelFilter::Info,
    };
    TermLogger::init(level_filter, Config::default()).unwrap();

    // Process subcommand:
    if matches.subcommand_matches("serve").is_some() {
        server::start_server();
    } else if matches.subcommand_matches("ping").is_some() {
        ping();
    } else if matches.subcommand_matches("list").is_some() {
        topic_list();
    } else if let Some(matches) = matches.subcommand_matches("pub") {
        let topic = matches.value_of("topic").unwrap().to_string();
        let value = matches.value_of("value").unwrap().to_string();
        publish(topic, value);
    } else if let Some(matches) = matches.subcommand_matches("echo") {
        let topic = matches.value_of("topic").unwrap().to_string();
        topic_echo(topic);
    } else {
        println!("No subcommand!");
    }
}

fn ping() {
    info!("PeNGG");

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let client_node = create_client().await.unwrap();
        info!("Created node {:?}", client_node);
    });
}

/// Publish a value to a topic.
fn publish(topic: String, value: String) {
    info!("publishing to topic");

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let client_node = create_client().await.unwrap();
        client_node.publish(topic, value).await.unwrap();

        client_node.bye().await.unwrap();
    });
}

fn topic_list() {
    info!("Listing all to topics");

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let client_node = create_client().await.unwrap();

        let topics = client_node.topic_list().await.unwrap();
        for topic in topics {
            println!("Topic: {}", topic);
        }
        client_node.bye().await.unwrap();
    });
}

fn topic_echo(topic: String) {
    info!("Echoing topic {}", topic);

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let client_node = create_client().await.unwrap();
        let mut values = client_node.subscribe(topic).await.unwrap();
        while let Some(value) = values.recv().await {
            info!("Got value: {}", value);
        }
    });
}
