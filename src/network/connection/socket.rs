use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, SystemTime};
use crate::network::connection::{Connection, Pipeline, Writable};
use crate::network::{ByteOrder, Error, PacketDirection, PacketState};
use crate::network::buffer::Buffer;

pub struct SocketConnection<'a> {
    packet_state: PacketState,
    pipeline: Pipeline<'a>,
    socket: TcpStream
}

impl<'a> Connection<'a, TcpStream> for SocketConnection<'a> {

    fn new(object: TcpStream, pipeline: Pipeline<'a>) -> Self {
        SocketConnection { socket: object, pipeline, packet_state: PacketState::Handshaking }
    }

    fn write<T: Writable>(&mut self, packet: T) -> Result<usize, Error> {
        let buffer = Buffer::empty(true, None);

        match packet.write(buffer) {
            Ok(buffer) => {
                match self.pipeline.encode(buffer) {
                    Ok(buffer) => {
                        match self.socket.write(&*buffer.to_bytes()) {
                            Ok(size) => {
                                self.socket.flush().unwrap();
                                Ok(size)
                            },
                            Err(error) => Err(Error::other(error.to_string()))
                        }
                    },
                    Err(error) => Err(Error::other(error.to_string()))
                }
            },
            Err(error) => Err(Error::other(error.to_string()))
        }
    }

    fn read_buffer(&mut self, timeout: Option<Duration>, order: ByteOrder) -> Result<(Buffer, Duration), Error> {
        let socket_timeout = self.get_timeout()?;

        if timeout.is_some() {
            self.set_timeout(timeout)?;
        }

        let mut read = [0; 1024];
        let mut bytes = Vec::new();
        let time = SystemTime::now();
        match self.socket.read(&mut read) {
            Ok(size) => {
                bytes = read[0..size].to_vec();
            },
            Err(error) => {
                if socket_timeout.is_some() {
                    self.set_timeout(socket_timeout)?;
                }
                return Err(Error::not_readable(error.to_string()));
            }
        }

        let reached_timeout = time.elapsed();
        if reached_timeout.is_err() {
            return Err(Error::other(reached_timeout.err().unwrap().to_string()));
        }
        let reached_timeout = reached_timeout.unwrap();
        if socket_timeout.is_some() {
            self.set_timeout(socket_timeout)?;
        }

        Ok((Buffer::new(bytes, true, Some(order)), reached_timeout))
    }

    fn state(&self) -> PacketState {
        self.packet_state
    }

    fn bound() -> PacketDirection {
        PacketDirection::Clientbound
    }

}

impl<'a> SocketConnection<'a> {
    pub fn set_timeout(&self, timeout: Option<Duration>) -> Result<(), Error> {
        match self.socket.set_read_timeout(timeout) {
            Err(error) => Err(Error::other(error.to_string())),
            Ok(()) => Ok(())
        }
    }

    pub fn get_timeout(&self) -> Result<Option<Duration>, Error> {
        let socket_timeout = self.socket.read_timeout();
        if socket_timeout.is_err() {
            return Err(Error::other(socket_timeout.err().unwrap().to_string()));
        }
        Ok(socket_timeout.unwrap())
    }
}