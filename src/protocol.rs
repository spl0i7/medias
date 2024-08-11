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
}

#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    Connect,
    Bind,
}

impl Command {
    pub fn to_bytes(&self) -> u8 {
        match self {
            Command::Connect => 1,
            Command::Bind => 2,
        }
    }

    pub fn from_bytes(input: &[u8]) -> Result<Self, Error> {
        if input.is_empty() {
            return Err(Error::UnrecognizedCommand);
        }
        match input[0] {
            1 => Ok(Command::Connect),
            2 => Ok(Command::Bind),
            _ => Err(Error::UnrecognizedCommand),
        }
    }
}

#[derive(Debug)]
pub enum Reply {
    RequestGranted,
    RequestRejected,
    NotRunningIdentd,
    CouldNotConfirmId,
}

impl Reply {
    pub fn to_bytes(&self) -> u8 {
        match self {
            Reply::RequestGranted => 0x5A,
            Reply::RequestRejected => 0x5B,
            Reply::NotRunningIdentd => 0x5C,
            Reply::CouldNotConfirmId => 0x5D,
        }
    }

    pub fn from_bytes(input: &[u8]) -> Result<Self, Error> {
        if input.is_empty() {
            return Err(Error::UnrecognizedReply);
        }

        match input[0] {
            0x5A => Ok(Reply::RequestGranted),
            0x5B => Ok(Reply::RequestRejected),
            0x5C => Ok(Reply::NotRunningIdentd),
            0x5D => Ok(Reply::CouldNotConfirmId),
            _ => Err(Error::UnrecognizedReply),
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub version: u8,
    pub command: Command,
    pub dest_port: u16,
    pub dest_ip: u32,
    pub id: Vec<u8>,
}

impl Request {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut raw = Vec::new();
        raw.push(self.version);

        raw.push(self.command.to_bytes());
        raw.extend(self.dest_port.to_be_bytes());
        raw.extend(self.dest_ip.to_be_bytes());
        raw.extend(&self.id);

        raw
    }
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

    pub fn ip_bytes(&self) -> (u8, u8, u8, u8) {
        let bytes = self.dest_ip.to_be_bytes();
        (bytes[0], bytes[1], bytes[2], bytes[3])
    }
}

pub struct Response {
    pub version: u8,
    pub reply: Reply,
    pub dest_port: u16,
    pub dest_ip: u32,
}

impl Response {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut raw = Vec::new();
        raw.push(self.version);
        raw.push(self.reply.to_bytes());
        raw.extend(self.dest_port.to_be_bytes());
        raw.extend(self.dest_ip.to_be_bytes());

        raw
    }
    pub fn from_bytes(input: &[u8]) -> Result<Self, Error> {
        if input.len() < 2 {
            return Err(Error::InvalidRequest);
        }

        let version = input[0];
        let reply = Reply::from_bytes(&input[1..])?;
        let dest_port = u16::from_be_bytes([input[2], input[3]]);
        let dest_ip = u32::from_be_bytes([input[4], input[5], input[6], input[7]]);

        Ok(Self {
            version,
            reply,
            dest_port,
            dest_ip,
        })
    }
}
