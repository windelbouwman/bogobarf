
mod server;
mod fancyheader;
mod client;
mod message;
mod peer;
mod connection;

#[macro_use]
extern crate log;
extern crate simplelog;
use simplelog::{TermLogger, Config, LevelFilter};
extern crate clap;
use clap::{App, SubCommand};

extern crate tokio;


fn main() {
    TermLogger::init(LevelFilter::Debug, Config::default()).unwrap();

    let matches = App::new("bogobarf")
       .version("0.0.0")
       .about("Robotic middleware")
       .subcommand(SubCommand::with_name("serve")
          .about("Starts the server"))
       .subcommand(SubCommand::with_name("ping")
          .about("Pings the server"))
       .subcommand(SubCommand::with_name("echo")
          .about("Echo a topic"))
       .subcommand(SubCommand::with_name("pub")
          .about("Publish data onto a topic"))
       .get_matches();

    if matches.subcommand_matches("serve").is_some() {
        server::start_server();
    } else if matches.subcommand_matches("ping").is_some() {
        client::ping();
    } else {
        println!("No subcommand!");

    }
}
