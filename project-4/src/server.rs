use super::api::{Request, Response};
// use crate::StoreResult;
use crate::{logger, Engine, StoreError, StoreResult, ThreadPool};

use slog::{error, o, Logger};
use std::io::Write;
use std::net::{TcpListener, TcpStream};

struct Server<E: Engine, P: ThreadPool> {
    engine: E,
    pool: P,
    logger: Logger,
}

impl<E: Engine, TP: ThreadPool> Server<E, TP> {
    pub fn new(e: E, tp: TP) -> Self {
        Server {
            engine: e,
            pool: tp,
            logger: logger("server"),
        }
    }

    pub fn run<A>(self, addr: A) -> StoreResult<()>
    where
        A: std::net::ToSocketAddrs,
    {
        let listner = TcpListener::bind(addr)?;

        for stream in listner.incoming() {
            let e = self.engine.clone();
            let l = self.logger.new(o!("module" => "stream"));

            self.pool.spawn(move || match stream {
                Ok(stream) => {
                    if let Err(err) = serve(e, stream) {
                        error!(&l, "serve: {}", err);
                    }
                }
                Err(err) => {
                    error!(&l, "connection failed: {}", err);
                }
            })
        }
        Ok(())
    }
}

fn serve<E>(engine: E, stream: TcpStream) -> StoreResult<()>
where
    E: Engine,
{
    let reader = std::io::BufReader::new(&stream);
    let requests = serde_json::Deserializer::from_reader(reader).into_iter::<Request>();
    let mut writer = std::io::BufWriter::new(&stream);

    macro_rules! send_resp {
        ($resp:expr) => {{
            let resp = $resp;
            serde_json::to_writer(&mut writer, &resp)?;
            writer.flush()?;
        };};
    }

    for request in requests {
        let req = request?;

        match req {
            Request::Get { key } => send_resp!(match engine.get(key) {
                Ok(value) => Response::Ok(value),
                Err(err) => Response::Err(format!("{}", err)),
            }),
            Request::Set { key, value } => send_resp!(match engine.set(key, value) {
                Ok(_) => Response::Ok(()),
                Err(err) => Response::Err(format!("{}", err)),
            }),
            Request::Remove { key } => send_resp!(match engine.remove(key) {
                Ok(_) => Response::Ok(()),
                Err(err) => Response::Err(format!("{}", err)),
            }),
        }
    }

    Ok(())
}
