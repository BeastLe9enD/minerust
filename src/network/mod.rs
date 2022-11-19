pub mod buffer;
pub mod connection;

use std::fmt::{Display, Formatter};
use crate::network::ErrorType::{IllegalPacket, NotReadable, NotWritable, Other, OutOfBounds};

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

#[derive(Debug)]
pub struct Error {
    _type: ErrorType
}

#[derive(Debug)]
pub enum ErrorType {
    OutOfBounds(usize, usize),
    NotWritable(String),
    NotReadable(String),
    IllegalPacket(i32, String),
    Other(String)
}

impl Display for ErrorType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OutOfBounds(length, overflow) => write!(formatter, "Out Of Bounds Error => You can't read more from this buffer! You reached the end of the buffer ({} > {})", length, overflow),
            NotWritable(message) => write!(formatter, "Not writable Error => Writable flag for {} is on false!", message),
            NotReadable(message) => write!(formatter, "Not writable Error => Readable flag for {} is on false!", message),
            IllegalPacket(packet_id, version) => write!(formatter, "Invalid Packet Error => No packet {} for the version {} available!", packet_id, version),
            Other(error_text) => write!(formatter, "{}", error_text)
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self._type)
    }
}

impl Error {

    fn out_of_bounds(length: usize, overflow: usize) -> Self {
        Self { _type: OutOfBounds(length, overflow) }
    }

    fn not_writable(message: &str) -> Self {
        Self { _type: NotWritable(String::from(message)) }
    }

    fn not_readable(message: String) -> Self {
        Self { _type: NotReadable(message) }
    }

    fn illegal_packet(packet_id: i32, version: String) -> Self {
        Self { _type: IllegalPacket(packet_id, version) }
    }

    fn other(error_text: String) -> Self {
        Self { _type: Other(error_text) }
    }

}

impl std::error::Error for Error {}

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