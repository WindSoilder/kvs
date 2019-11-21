use super::Response;
use crate::command::Instruction;
use crate::error::Result;
use serde_json;
use std::io::prelude::*;
use std::net::TcpStream;

pub struct Client {
    inner: TcpStream,
}

impl Client {
    pub fn connect(addr: &str) -> Result<Client> {
        let stream: TcpStream = TcpStream::connect(addr)?;
        Ok(Client { inner: stream })
    }

    pub fn send_instruction(&mut self, inst: &Instruction) -> Result<()> {
        let out: String = serde_json::to_string(inst)?;
        self.inner.write(out.as_bytes())?;
        Ok(())
    }

    pub fn read_response(&mut self) -> Result<Response> {
        let mut buffer: [u8; 1024] = [0; 1024];
        let bytes = self.inner.read(&mut buffer)?;
        let response: Response = serde_json::from_slice(&buffer[..bytes])?;
        Ok(response)
    }
}
