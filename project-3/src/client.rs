use crate::{Error as KvsError, GetResponse, RemoveResponse, Request, Result, SetResponse};
use serde_json::de::{Deserializer, IoRead};
use std::io::{BufReader, BufWriter, Write};
use std::net::TcpStream;

use serde::Deserialize;

pub struct Client {
    reader: Deserializer<IoRead<BufReader<TcpStream>>>,
    writer: BufWriter<TcpStream>,
}

impl Client {
    pub fn connect<T: std::net::ToSocketAddrs>(addr: T) -> Result<Self> {
        let tcp_stream_reader = TcpStream::connect(addr)?;
        let tcp_stream_writer = tcp_stream_reader.try_clone()?;

        let reader = Deserializer::from_reader(BufReader::new(tcp_stream_reader));
        let writer = BufWriter::new(tcp_stream_writer);

        Ok(Client { reader, writer })
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        serde_json::to_writer(&mut self.writer, &Request::Get { key })?;
        self.writer.flush()?;

        let resp: GetResponse = GetResponse::deserialize(&mut self.reader)?;

        match resp {
            GetResponse::Ok(data) => Ok(data),
            GetResponse::Err(message) => Err(KvsError::WithMessage(message)),
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        serde_json::to_writer(&mut self.writer, &Request::Set { key, value })?;
        self.writer.flush()?;

        match SetResponse::deserialize(&mut self.reader)? {
            SetResponse::Ok(data) => Ok(data),
            SetResponse::Err(message) => Err(KvsError::WithMessage(message)),
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        serde_json::to_writer(&mut self.writer, &Request::Remove { key })?;
        self.writer.flush()?;

        match RemoveResponse::deserialize(&mut self.reader)? {
            RemoveResponse::Ok(data) => Ok(data),
            RemoveResponse::Err(message) => Err(KvsError::WithMessage(message)),
        }
    }
}
