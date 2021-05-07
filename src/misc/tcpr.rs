use std::{
    cell::RefCell,
    io::{
        prelude::{BufRead, Write},
        BufReader, Read,
    },
    net::{Shutdown, TcpStream},
    thread,
    thread::sleep,
    time::Duration,
};

pub struct Server {
    socket: TcpStream,
}

impl Server {
    pub fn new(address: &str, password: Option<&String>) -> Result<Server, String> {
        let mut server = {
            let socket = TcpStream::connect(address);

            if let Err(error) = socket {
                return Err(format!("Error connecting to {}\nError: {}", address, error));
            }

            Server {
                socket: socket.unwrap(),
            }
        };

        // TODO: Check that password was correct (it assumes it is currently)
        if let Some(pass) = password {
            if let Err(error) = server.send_command(&pass) {
                return Err(error);
            }
        }

        Ok(server)
    }

    pub fn send_command(&mut self, cmd: &str) -> Result<(), String> {
        match self.socket.write(cmd.as_bytes()) {
            Err(err) => Err(err.to_string()),
            _ => Ok(()),
        }
    }

    pub fn wait_for_result(&mut self) -> Result<String, String> {
        let mut content = String::new();

        // Read the next 100 messages before forcefully ending
        for n in 0..100 {
            let size: usize = match self.socket.read_to_string(&mut content) {
                Err(error) => {
                    return Err(error.to_string());
                }
                Ok(size) => size,
            };

            if content.starts_with("[R_STAT]") {
                return Ok(content);
            }
        }

        Err("No response given".to_string())
    }

    // TODO: Clean this up (does not give the actual error back)
    pub fn send_and_read(&mut self, cmd: &str) -> Result<String, String> {
        if self.send_command(&cmd).is_ok() {
            if let Ok(msg) = self.wait_for_result() {
                return Ok(msg);
            }
        }

        Err("No response given".to_string())
    }
}
