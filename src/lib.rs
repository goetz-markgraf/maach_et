#![allow(unused)]

use std::fmt;

pub mod chat;
pub mod config;
pub mod llm_api;
pub mod tools;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Llm(String),
    ChatControl(String),
    Git(String),
    Tool(String),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Llm(msg) => write!(f, "LLM error: {}", msg),
            Error::ChatControl(msg) => write!(f, "Chat control error: {}", msg),
            Error::Git(msg) => write!(f, "Error while processing git command: {}", msg),
            Error::Tool(msg) => write!(f, "Tool error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}
