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
use env_logger::Target;
use kvs::thread_pool::{self, NaiveThreadPool, ThreadPool};
use kvs::{Engine, KvStore, Result, Server, SledKvsEngine};
use log::info;
use log::LevelFilter;
use std::path::Path;
use std::process;
use std::str::FromStr;

fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .target(Target::Stderr)
        .init();
    info!(
        "Kvs server start up, version is: {}",
        env!("CARGO_PKG_VERSION")
    );
    let app: App = App::new("kvs_server")
        .version(env!("CARGO_PKG_VERSION"))
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
    let addr: &str = matches.value_of("addr").unwrap_or("127.0.0.1:4000");
    let engine: Engine = Engine::from_str(matches.value_of("engine").unwrap_or("kvs"))?;

    let pool = thread_pool::NaiveThreadPool::new(10)?;
    // other engine already running?
    match engine {
        Engine::Kvs => {
            if SledKvsEngine::db_exists(Path::new(".")) {
                eprintln!("Kvs engine already run in the current folder.");
                process::exit(1);
            }
        }
        Engine::Sled => {
            if KvStore::db_exists(Path::new(".")) {
                eprintln!("Sled engine already run in the current folder.");
                process::exit(1);
            }
        }
    };

    info!("Listening on {}", addr);
    info!("Using engine {:?}", engine);

    match engine {
        Engine::Kvs => {
            let mut server: Server<KvStore, NaiveThreadPool> =
                Server::new(addr, KvStore::open(Path::new("."))?, pool)?;
            server.serve_forever()?;
        }
        Engine::Sled => {
            let mut server: Server<SledKvsEngine, NaiveThreadPool> =
                Server::new(addr, SledKvsEngine::open(Path::new("."))?, pool)?;
            server.serve_forever()?;
        }
    }
    Ok(())
}
