use super::Response;
use crate::command::Instruction;
use crate::engine::KvsEngine;
use crate::error::Result;
use crate::thread_pool::ThreadPool;
use log::{debug, error};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

pub struct Server<E: KvsEngine, P: ThreadPool> {
    listener: TcpListener,
    engine: E,
    thread_pool: P,
}

impl<E: KvsEngine, P: ThreadPool> Server<E, P> {
    pub fn new<T>(addr: T, engine: E, thread_pool: P) -> Result<Server<E, P>>
    where
        T: ToSocketAddrs,
    {
        Ok(Server {
            listener: TcpListener::bind(addr)?,
            engine,
            thread_pool,
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
                    let engine_work = self.engine.clone();
                    self.thread_pool.spawn(move || {
                        Self::handle_client(client_stream, &engine_work).unwrap();
                    })
                }
                Err(e) => error!("Connection failed, reason: {:?}", e),
            }
        }
        Ok(())
    }

    pub fn handle_client(mut client_stream: TcpStream, engine: &impl KvsEngine) -> Result<()> {
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
                let response: Response = Self::execute_instruction(instruction, engine);
                // TODO: here we need to check serde_json result..  What if it goes into fail..
                let bytes: Vec<u8> = serde_json::to_vec(&response)?;
                client_stream.write(&bytes)?;
                debug!("Solve complete for peer: {}", peer_addr);
            }
        }
        Ok(())
    }

    fn execute_instruction(instruction: Instruction, engine: &impl KvsEngine) -> Response {
        match instruction {
            Instruction::Set { key, value } => {
                let result = engine.set(key, value);
                match result {
                    Ok(_) => Response::new_ok(),
                    Err(e) => Response::new_err(e.to_string()),
                }
            }
            Instruction::Get { key } => {
                let result = engine.get(key);
                match result {
                    Ok(Some(s)) => Response::new_ok_with_body(s),
                    Ok(None) => Response::new_err(String::from("Key not found")),
                    Err(e) => Response::new_err(String::from(e.to_string())),
                }
            }
            Instruction::Rm { key } => {
                let result = engine.remove(key);
                match result {
                    Ok(_) => Response::new_ok(),
                    Err(e) => Response::new_err(e.to_string()),
                }
            }
        }
    }
}
