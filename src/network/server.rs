use super::Response;
use crate::command::Instruction;
use crate::engine::{Engine, KvsEngine};
use crate::error::Result;
use log::{debug, error};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

pub struct Server {
    listener: TcpListener,
    store: Box<dyn KvsEngine>,
}

impl Server {
    pub fn new<T>(addr: T, engine: Box<dyn KvsEngine>) -> Result<Server>
    where
        T: ToSocketAddrs,
    {
        Ok(Server {
            listener: TcpListener::bind(addr)?,
            store: engine,
        })
    }

    pub fn serve_forever(&mut self) -> Result<()> {
        debug!("Waiting for connections...");
        for stream in self.listener.incoming() {
            match stream {
                Ok(client_stream) => {
                    debug!(
                        "New connection established from {}",
                        client_stream.peer_addr()?
                    );
                    Self::handle_client(client_stream, &mut self.store)?;
                }
                Err(e) => error!("Connection failed, reason: {:?}", e),
            }
        }
        Ok(())
    }

    pub fn handle_client(
        mut client_stream: TcpStream,
        store: &mut Box<dyn KvsEngine>,
    ) -> Result<()> {
        let peer_addr = client_stream.peer_addr()?;
        debug!("Waiting data from {}", peer_addr);

        let mut buf: [u8; 1024] = [0; 1024];
        loop {
            let bytes: usize = client_stream.read(&mut buf)?;
            if bytes == 0 {
                debug!(
                    "Connection closed by peer {}, so this connection is closed.",
                    peer_addr
                );
                break;
            } else {
                debug!("Receive data from peer: {}", peer_addr);
                let user_command: String = String::from_utf8_lossy(&buf[..bytes]).into_owned();
                let instruction: Instruction = serde_json::from_str(&user_command)?;
                debug!("Peer: {}, Instruction: {:?}", peer_addr, instruction);
                // handle for user request.
                let response: Response = Self::execute_instruction(instruction, store);
                // TODO: here we need to check serde_json result..  What if it goes into fail..
                let bytes: Vec<u8> = serde_json::to_vec(&response)?;
                client_stream.write(&bytes)?;
                debug!("Solve complete for peer: {}", peer_addr);
            }
        }
        Ok(())
    }

    fn execute_instruction(instruction: Instruction, store: &mut Box<dyn KvsEngine>) -> Response {
        match instruction {
            Instruction::Set { key, value } => {
                let result = (*store).set(key, value);
                match result {
                    Ok(_) => Response::new_ok(),
                    Err(e) => Response::new_err(e.to_string()),
                }
            }
            Instruction::Get { key } => {
                let result = (*store).get(key);
                match result {
                    Ok(Some(s)) => Response::new_ok_with_body(s),
                    Ok(None) => Response::new_err(String::from("Key not found")),
                    Err(e) => Response::new_err(String::from(e.to_string())),
                }
            }
            Instruction::Rm { key } => {
                let result = (*store).remove(key);
                match result {
                    Ok(_) => Response::new_ok(),
                    Err(e) => Response::new_err(e.to_string()),
                }
            }
        }
    }
}
