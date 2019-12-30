use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
/// Kvs server response.
pub struct Response {
    /// Status code, it's like http response code.
    status: Status,
    /// Relative message.
    message: String,
    /// Response body.
    body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    OK,
    ERROR,
}

impl Response {
    pub fn new(status: Status, message: String, body: String) -> Response {
        Response {
            status,
            message,
            body,
        }
    }

    pub fn new_ok() -> Response {
        Response {
            status: Status::OK,
            message: String::from(""),
            body: String::from(""),
        }
    }

    pub fn new_ok_with_body(body: String) -> Response {
        Response {
            status: Status::OK,
            message: String::from(""),
            body,
        }
    }
    pub fn new_err(message: String) -> Response {
        Response {
            status: Status::ERROR,
            message,
            body: String::from(""),
        }
    }

    pub fn is_ok(&self) -> bool {
        match self.status {
            Status::OK => true,
            _ => false,
        }
    }

    pub fn get_message(&self) -> &String {
        &self.message
    }

    pub fn get_body(&self) -> &String {
        &self.body
    }
}
