use crate::{Engine, Request, Result};
use serde_json::Deserializer;
use std::{
    io::{BufReader, BufWriter},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

pub struct Server<T: Engine> {
    engine: T,
}

impl<T: Engine> Server<T> {
    fn new(engine: T) -> Self {
        Server { engine }
    }

    fn run<A: ToSocketAddrs>(&mut self, addr: A) -> Result<()> {
        let listener = TcpListener::bind(addr)?;

        for stream in listener.incoming() {
            match stream {
                Ok(tcp_stream) => {
                    if let Err(err) = self.serve(tcp_stream) {
                        error!("serving failed: {}", err)
                    }
                }
                Err(err) => {
                    error!("connection failed: {}", err);
                }
            }
        }

        Ok(())
    }

    fn serve(&mut self, stream: TcpStream) -> Result<()> {
        let peer = stream.peer_addr()?;
        let reader = BufReader::new(&stream);
        let mut writer = BufWriter::new(&stream);
        let request_reader = Deserializer::from_reader(reader).into_iter::<Request>();

        for request in request_reader {
            let request = request?;

            match request {
                Request::Get { key } => {}
                Request::Set { key, value } => {}
                Request::Remove { key } => {}
            }
        }

        Ok(())
    }
}
