use project_3::{Client, Result};
use std::process::exit;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "kvs-client")]
struct Opt {
    #[structopt(flatten)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "get", about = "get the string value of a given string key")]
    Get {
        #[structopt(name = "KEY")]
        key: String,
        #[structopt(long, value_name = "IP:PORT", default_value = "127.0.0.1:4000")]
        addr: String,
    },
    #[structopt(name = "set", about = "set the value of a string key to a string")]
    Set {
        #[structopt(name = "KEY")]
        key: String,
        #[structopt(name = "VALUE")]
        value: String,
        #[structopt(long, value_name = "IP:PORT", default_value = "127.0.0.1:4000")]
        addr: String,
    },
    #[structopt(name = "rm", about = "remove a given string key")]
    Remove {
        #[structopt(name = "KEY")]
        key: String,
        #[structopt(long, value_name = "IP:PORT", default_value = "127.0.0.1:4000")]
        addr: String,
    },
}

fn main() {
    let opt = Opt::from_args();
    if let Err(e) = run(opt) {
        eprintln!("{}", e);
        exit(1);
    }
}

fn run(opt: Opt) -> Result<()> {
    match opt.command {
        Command::Get { key, addr } => {
            let mut client = Client::connect(addr)?;

            if let Some(value) = client.get(key)? {
                println!("{}", value)
            } else {
                println!("Key not found")
            }
        }
        Command::Set { key, value, addr } => Client::connect(addr)?.set(key, value)?,
        Command::Remove { key, addr } => Client::connect(addr)?.remove(key)?,
    }
    Ok(())
}
