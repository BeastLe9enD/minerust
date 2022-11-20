pub mod buffer;
pub mod connection;

use std::{
    fmt::{Display, Formatter},
    io
};

use thiserror::Error;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ByteOrder {
    BigEndian,
    LittleEndian
}

impl ByteOrder {
    #[inline]
    pub fn system_order() -> Self {
        if cfg!(target_endian = "little") {
            ByteOrder::LittleEndian
        } else {
            ByteOrder::BigEndian
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Out Of Bounds Error => You can't read more from this buffer! You reached the end of the buffer ({0} > {1})")]
    OutOfBounds(usize, usize),
    #[error("Not writable Error => Writable flag for {0} is on false!")]
    NotWritable(String),
    #[error("Not writable Error => Readable flag for {0} is on false!")]
    NotReadable(String),
    #[error("Invalid Packet Error => No packet {0} for the version {1} available!")]
    IllegalPacket(i32, String),
    #[error("{0}")]
    Other(String),
    #[error("Io Error: {0}")]
    IoError(#[from] io::Error)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketDirection {
    Clientbound,
    Serverbound
}

impl Display for PacketDirection {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PacketDirection::Serverbound => write!(formatter, "Serverbound"),
            PacketDirection::Clientbound => write!(formatter, "Clientbound")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketState {
    Handshaking,
    Login,
    Status,
    Play
}

impl Display for PacketState {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PacketState::Handshaking => write!(formatter, "Handshaking"),
            PacketState::Login => write!(formatter, "Login"),
            PacketState::Status => write!(formatter, "Status"),
            PacketState::Play => write!(formatter, "Play")
        }
    }
}

pub trait ProtocolVersion {
    fn id() -> i32;
    fn literal() -> &'static str;
}
