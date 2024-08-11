#![allow(dead_code)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid protocol request command")]
    UnrecognizedCommand,
    #[error("invalid protocol reply kind")]
    UnrecognizedReply,
    #[error("invalid protocol request")]
    InvalidRequest,
    #[error("io error")]
    IO(#[from] tokio::io::Error),
    #[error("connection dial timeout")]
    Timeout(#[from] tokio::time::error::Elapsed),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    Connect,
    Bind,
}

impl Command {
    pub const fn to_bytes(&self) -> u8 {
        match self {
            Self::Connect => 1,
            Self::Bind => 2,
        }
    }

    pub const fn from_bytes(input: &[u8]) -> Result<Self, Error> {
        if input.is_empty() {
            return Err(Error::UnrecognizedCommand);
        }
        match input[0] {
            1 => Ok(Self::Connect),
            2 => Ok(Self::Bind),
            _ => Err(Error::UnrecognizedCommand),
        }
    }
}

#[derive(Debug, Default)]
pub enum Reply {
    #[default]
    RequestGranted,
    RequestRejected,
    NotRunningIdentd,
    CouldNotConfirmId,
}

impl Reply {
    pub const fn to_bytes(&self) -> u8 {
        match self {
            Self::RequestGranted => 0x5A,
            Self::RequestRejected => 0x5B,
            Self::NotRunningIdentd => 0x5C,
            Self::CouldNotConfirmId => 0x5D,
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub(crate) version: u8,
    pub(crate) command: Command,
    pub(crate) dest_port: u16,
    pub(crate) dest_ip: u32,
    pub(crate) id: Vec<u8>,
}

impl Request {
    pub fn from_bytes(input: &[u8]) -> Result<Self, Error> {
        if input.len() < 8 {
            return Err(Error::InvalidRequest);
        }

        let version = input[0];
        let command = Command::from_bytes(&input[1..])?;
        let dest_port = u16::from_be_bytes([input[2], input[3]]);
        let dest_ip = u32::from_be_bytes([input[4], input[5], input[6], input[7]]);
        let id = input[8..].to_vec();

        Ok(Self {
            version,
            command,
            dest_port,
            dest_ip,
            id,
        })
    }
}

#[derive(Debug, Default)]
pub struct Response {
    pub(crate) version: u8,
    pub(crate) reply: Reply,
    pub(crate) dest_port: u16,
    pub(crate) dest_ip: u32,
}

impl Response {
    pub fn reject_response() -> Self {
        Response {
            version: 0,
            reply: Reply::RequestRejected,
            dest_port: 0,
            dest_ip: 0,
        }
    }
    pub fn to_bytes(&self) -> [u8; 8] {
        let mut raw = [0; 8];

        raw[0] = self.version;
        raw[1] = self.reply.to_bytes();
        raw[2..4].copy_from_slice(&self.dest_port.to_be_bytes());
        raw[4..8].copy_from_slice(&self.dest_ip.to_be_bytes());

        raw
    }
}
