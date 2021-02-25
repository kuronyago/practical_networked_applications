// use anyhow::{Context, Result};
use project_4::{StoreError, StoreResult};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    #[structopt(long)]
    addr: std::net::SocketAddr,
}

impl Options {
    fn run(self) -> StoreResult<()> {
        todo!()
    }
}

fn main() -> StoreResult<()> {
    println!("Hello, world!");

    Ok(())
}
