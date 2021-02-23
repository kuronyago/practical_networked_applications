use crate::{Engine, GetResponse, RemoveResponse, Request, Result, SetResponse};
use serde::Serialize;
use serde_json::Deserializer;
use slog::{debug, error, info, Logger};
use std::thread;
use std::{
    io::{BufReader, BufWriter},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

pub struct Server<T: Engine> {
    engine: T,
    logger: Logger,
}

impl<T: Engine> Server<T> {
    pub fn new(engine: T, logger: Logger) -> Self {
        Server { engine, logger }
    }

    pub fn run<A: ToSocketAddrs>(mut self, addr: A) -> Result<()> {
        let listener = TcpListener::bind(addr)?;

        for stream in listener.incoming() {
            match stream {
                Ok(tcp_stream) => {
                    if let Err(err) = self.serve(tcp_stream) {
                        error!(self.logger, "serving failed: {}", err)
                    }
                }
                Err(err) => {
                    error!(self.logger, "connection failed: {}", err);
                }
            }
        }

        Ok(())
    }

    fn serve(&mut self, stream: TcpStream) -> Result<()> {
        let reader = BufReader::new(&stream);
        let mut writer = BufWriter::new(&stream);
        let request_reader = Deserializer::from_reader(reader).into_iter::<Request>();

        for request in request_reader {
            info!(self.logger, "current thread: {:?}", thread::current().id());

            let request = request?;

            match request {
                Request::Get { key } => {
                    let resp: GetResponse = {
                        match self.engine.get(key) {
                            Ok(value) => GetResponse::Ok(value),
                            Err(err) => GetResponse::Err(format!("{}", err)),
                        }
                    };

                    send_response(&mut writer, resp)?;
                }
                Request::Set { key, value } => {
                    let resp: SetResponse = {
                        match self.engine.set(key, value) {
                            Ok(_) => SetResponse::Ok(()),
                            Err(err) => SetResponse::Err(format!("{}", err)),
                        }
                    };

                    send_response(&mut writer, resp)?;
                }
                Request::Remove { key } => {
                    let resp: RemoveResponse = {
                        match self.engine.remove(key) {
                            Ok(_) => RemoveResponse::Ok(()),
                            Err(err) => RemoveResponse::Err(format!("{}", err)),
                        }
                    };

                    send_response(&mut writer, resp)?;
                }
            };
        }

        Ok(())
    }
}

fn send_response<W, S>(writer: &mut W, value: S) -> std::io::Result<()>
where
    W: std::io::Write,
    S: Serialize,
{
    let mut writer = writer;
    serde_json::to_writer(&mut writer, &value)?;
    writer.flush()
}
