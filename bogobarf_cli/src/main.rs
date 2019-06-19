
#[macro_use]
extern crate log;
extern crate simplelog;
use simplelog::{Config, LevelFilter, TermLogger};
extern crate clap;
use clap::{App, Arg, SubCommand};

extern crate tokio;
use tokio::prelude::*;

// crate bogobarf;
use bogobarf::create_client;
use bogobarf::server;

fn main() {
    let matches = App::new("bogobarf")
        .version("0.0.2")
        .about("Robotic middleware")
        .arg(Arg::with_name("v")
                               .short("v")
                               .multiple(true)
                               .help("Sets the level of verbosity"))
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

    let client = create_client().map(|_| ());
    tokio::run(client);
}

/// Publish a value to a topic.
fn publish(topic: String, value: String) {
    info!("publishing to topic");

    let task = create_client().and_then(|client_node| {
        client_node
            .publish(topic, value)
            .and_then(move |_| client_node.bye())
    });
    tokio::run(task);
}

fn topic_list() {
    info!("Listing all to topics");

    let task = create_client().and_then(|client_node| {
        client_node.topic_list().and_then(move |topics| {
            for topic in topics {
                println!("Topic: {}", topic);
            }
            client_node.bye()
        })
    });
    tokio::run(task);
}

fn topic_echo(topic: String) {
    info!("Echoing topic {}", topic);

    let task = create_client().and_then(|client_node| {
        client_node
            .subscribe(topic)
            .for_each(|value| {
                info!("Got value: {}", value);
                Ok(())
            })
            .map_err(|e| {
                println!("Error: {:?}", e);
            })
    });
    tokio::run(task);
}
