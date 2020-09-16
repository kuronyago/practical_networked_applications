#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;

use project_3::{
    Error as KvsError, Result as KvsResult, Server as KvsServer, Sled as KvsSled, Store,
};
use std::env::current_dir;
use std::net::SocketAddr;
use structopt::StructOpt;

const DEFAULT_ENGINE: Engine = Engine::Kvs;

#[derive(StructOpt)]
#[structopt(name = "kvs-server")]
struct Opt {
    #[structopt(long, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
    #[structopt(long)]
    engine: Option<Engine>,
}

impl Opt {
    fn run(self) -> KvsResult<()> {
        let engine = self.engine.unwrap_or(DEFAULT_ENGINE);
        info!("kvs-server {}", env!("CARGO_PKG_VERSION"));
        info!("Storage engine: {}", engine);
        info!("Listening on {}", self.addr);

        let write_path = current_dir()?.join("engine");
        let write_contents = format!("{}", engine);
        std::fs::write(write_path, write_contents)?;
        let path = current_dir()?;

        match engine {
            Engine::Kvs => {
                let kvs_engine = Store::open(path)?;
                KvsServer::new(kvs_engine).run(self.addr)
            }
            Engine::Sled => {
                let db = sled::open(path)?;
                let kvs_engine = KvsSled::new(db);
                KvsServer::new(kvs_engine).run(self.addr)
            }
        }
    }
}

arg_enum! {
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    enum Engine {
        Kvs,
        Sled,
    }
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let mut opt: Opt = Opt::from_args();

    let res: Result<(), KvsError> = {
        let wrapped_engine = current_engine();

        match wrapped_engine {
            Ok(engine) => {
                if opt.engine.is_none() {
                    opt.engine = engine;
                }

                if engine.is_some() && opt.engine != engine {
                    error!("wrong engine");
                    std::process::exit(1);
                }

                opt.run()
            }
            Err(err) => {
                error!("engine error: {}", err);
                Err(err)
            }
        }
    };

    if let Err(err) = res {
        error!("{}", err);
        std::process::exit(1);
    }
    info!("stop!");
}

fn current_engine() -> KvsResult<Option<Engine>> {
    let engine_path = current_dir()?.join("engine");

    if !engine_path.exists() {
        return Ok(None);
    }

    match std::fs::read_to_string(engine_path)?.parse() {
        Ok(engine) => Ok(Some(engine)),
        Err(err) => {
            warn!("invalid engine file: {}", err);
            Ok(None)
        }
    }
}
