//! The kvs-server executable supports the following command line arguments:
//!     kvs-server [--addr IP-PORT] [--engine ENGINE-NAME]
//!     Start the server and begin listening for incoming connections. --addr accepts an IP address, either v4 or v6, and a port number, with the format IP:PORT.
//! If --addr is not specified then listen on 127.0.0.1:4000.
//! If --engine is specified, then ENGINE-NAME must be either "kvs", in which case the built-in engine is used, or "sled", in which case sled is used.
//! If this is the first run (there is no data previously persisted) then the default value is "kvs"; if there is previously persisted data then the default is the engine already in use.
//! If data was previously persisted with a different engine than selected, print an error and exit with a non-zero exit code.
//! Print an error and return a non-zero exit code on failure to bind a socket, if ENGINE-NAME is invalid, if IP-PORT does not parse as an address.
//!     kvs-server -V
//!     Print the version.

use clap::{App, Arg};
use kvs::{Engine, Result};
use log::info;
use std::str::FromStr;

fn main() -> Result<()> {
    env_logger::init();
    info!("Kvs server start up.");
    let app: App = App::new("kvs_server")
        .arg(
            Arg::with_name("addr")
                .help("address to listen")
                .long("addr")
                .value_name("IP-PORT")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("engine")
                .help("storage engine to use")
                .long("engine")
                .value_name("ENGINE-NAME")
                .takes_value(true),
        );

    let matches = app.get_matches();
    let addr: &str = matches.value_of("addr").unwrap_or("localhost:4000");
    let engine: Engine = Engine::from_str(matches.value_of("engine").unwrap_or("kvs"))?;

    info!("Listening on {}", addr);
    info!("Using engine {:?}", engine);

    Ok(())
}
