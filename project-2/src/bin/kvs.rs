use clap::{App, Arg, SubCommand};
use kvs::{KvStore, KvStoreError, Result};
use std::env::current_dir;
use std::process::exit;

fn main() -> Result<()> {
    let app: clap::ArgMatches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .setting(clap::AppSettings::VersionlessSubcommands)
        .subcommand(SubCommand::with_name("get").arg(Arg::with_name("KEY").required(true)))
        .subcommand(
            SubCommand::with_name("set")
                .arg(Arg::with_name("KEY").required(true))
                .arg(Arg::with_name("VALUE").required(true)),
        )
        .subcommand(SubCommand::with_name("rm").arg(Arg::with_name("KEY").required(true)))
        .get_matches();

    match app.subcommand() {
        ("set", Some(matches)) => {
            let key: &str = matches.value_of("KEY").expect("KEY argument missing");
            let value: &str = matches.value_of("VALUE").expect("VALUE argument missing");
            let path = current_dir()?;
            let mut store: KvStore = KvStore::open(&path)?;
            store.set(key.to_string(), value.to_string())?;
        }

        ("get", Some(matches)) => {
            let key: &str = matches.value_of("KEY").expect("KEY argument missing");
            let path = current_dir()?;
            let mut store: KvStore = KvStore::open(&path)?;

            if let Some(value) = store.get(key.to_owned())? {
                println!("{}", value);
            } else {
                println!("Key not found");
            }
        }

        ("rm", Some(matches)) => {
            let key: &str = matches.value_of("KEY").expect("KEY argument missing");
            let path = current_dir()?;
            let mut store: KvStore = KvStore::open(&path)?;
            match store.remove(key.to_owned()) {
                Ok(()) => {}
                Err(KvStoreError::KeyNotFound) => {
                    println!("Key not found");
                    exit(1);
                }
                Err(e) => return Err(e),
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
