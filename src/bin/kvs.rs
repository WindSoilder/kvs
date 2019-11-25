use clap::{App, Arg, SubCommand};
use kvs::{KvStore, Repr, Result, KvsEngine};

use std::path::Path;
use std::process;

/// Basic behavior:
/// "set"
///   - The user invokes `kvs set mykey myvalue`
///   - `kvs` creates a value representing the "set" command, containing its key and value.
///   - It then serializes that command to a `String`.
///   - It then appends the serialized command to a file containing the log.
///   - If that succeeds, it exits silently with error code 0.
///   - If it fails, it exits by printing the error and returning a non-zero error code.
/// "get"
///   - The user invokes `kvs get mykey`
///   - `kvs` reads the entire log, one command at a time, recording the affected key and
///     file offset of the command to an in-memory key -> log pointer map.
///   - It then checks the map for the log pointer.
///   - If it fails, it prints "Key not found", and exits with exit code 0.
///   - If it succeeds:
///      - It deserializes the command to get the last recorded value of the key.
///      - It prints the value to stdout and exits with exit code 0.
/// "rm"
///   - The user invokes `kvs rm mykey`.
///   - Same as the "get" command, `kvs` reads the entire log to build the in-memory index.
///   - It then checks the map if the given key exists.
///   - If the key does not exist, it prints "Key not found", and exits with a non-zero error
///     code.
///   - If it succeeds:
///      - It creates a value representing the "rm" command, containing its key.
///      - It then appends the serialized command to the log.
///      - If that succeeds, it exits silently with error code 0.
///
/// The log is a record of the transactions committed to the database.  By "replying" the records
/// in the log on startup we reconstruct the previous state of the database.
fn get_app() -> App<'static, 'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            SubCommand::with_name("set")
                .about("Make a key associate with a value")
                .arg(
                    Arg::with_name("key")
                        .help("key to storage")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("value")
                        .help("relative value")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Get a value from key")
                .arg(
                    Arg::with_name("key")
                        .help("key to retrieve")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("Remove a key from storage engine")
                .arg(
                    Arg::with_name("key")
                        .help("key to remove")
                        .takes_value(true)
                        .required(true),
                ),
        )
}

fn main() -> Result<()> {
    // let mut store = KvStore::open(Path::new("kvs.db"))?;
    // Ok(())
    let app: App = get_app();
    let matches = app.get_matches();

    // Handle subcommands.
    // Create directory for kvs_db if the directory is not exists.
    let dir_name: &str = ".";
    let db_folder: &Path = Path::new(dir_name);
    let mut store = KvStore::open(db_folder)?;

    if let Some(sub_matches) = matches.subcommand_matches("set") {
        let key: &str = sub_matches.value_of("key").unwrap();
        let value: &str = sub_matches.value_of("value").unwrap();
        do_set(&mut store, key, value)?;
    } else if let Some(sub_matches) = matches.subcommand_matches("get") {
        let key: &str = sub_matches.value_of("key").unwrap();

        match do_get(&mut store, key) {
            Err(_) => {
                println!("Key not found");
            }
            Ok(Some(s)) => {
                println!("{}", s);
            }
            Ok(None) => {
                println!("Key not found");
            }
        }
    } else if let Some(sub_matches) = matches.subcommand_matches("rm") {
        let key: &str = sub_matches.value_of("key").unwrap();

        if let Err(e) = do_remove(&mut store, key) {
            match e.repr() {
                Repr::CommandError(_) => {
                    println!("Key not found");
                    process::exit(1);
                }
                _ => return Err(e),
            }
        }
    } else {
        process::exit(1);
    }
    Ok(())
}

/// execute kvs set command
fn do_set(store: &mut KvStore, key: &str, value: &str) -> Result<()> {
    store.set(String::from(key), String::from(value))
}

/// execute kvs remove command
fn do_remove(store: &mut KvStore, key: &str) -> Result<()> {
    store.remove(String::from(key))
}

/// execute kvs get command
fn do_get(store: &mut KvStore, key: &str) -> Result<Option<String>> {
    store.get(String::from(key))
}
