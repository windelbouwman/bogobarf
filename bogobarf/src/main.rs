mod client;
mod connection;
mod fancyheader;
mod message;
mod peer;
mod server;

#[macro_use]
extern crate log;
extern crate simplelog;
use simplelog::{Config, LevelFilter, TermLogger};
extern crate clap;
use clap::{App, Arg, SubCommand};

extern crate tokio;

fn main() {
    TermLogger::init(LevelFilter::Debug, Config::default()).unwrap();

    let matches = App::new("bogobarf")
        .version("0.0.0")
        .about("Robotic middleware")
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

    if matches.subcommand_matches("serve").is_some() {
        server::start_server();
    } else if matches.subcommand_matches("ping").is_some() {
        client::ping();
    } else if matches.subcommand_matches("list").is_some() {
        client::topic_list();
    } else if let Some(matches) = matches.subcommand_matches("pub") {
        let topic = matches.value_of("topic").unwrap().to_string();
        let value = matches.value_of("value").unwrap().to_string();
        client::publish(topic, value);
    } else if let Some(matches) = matches.subcommand_matches("echo") {
        let topic = matches.value_of("topic").unwrap().to_string();
        client::echo(topic);
    } else {
        println!("No subcommand!");
    }
}
