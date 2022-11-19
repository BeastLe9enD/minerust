use std::mem::size_of;
use std::time::Duration;
use uuid::Uuid;
use crate::network::buffer::Buffer;
use crate::network::{ByteOrder, Error, PacketDirection, PacketState};

pub mod pipeline;
pub mod socket;

pub trait Connection<'a, S> {
    fn new(object: S, pipeline: Pipeline<'a>) -> Self;

    fn write<T: Writable>(&mut self, packet: T) -> Result<usize, Error>;
    fn read_buffer(&mut self, timeout: Option<Duration>, order: ByteOrder) -> Result<(Buffer, Duration), Error>;

    fn state(&self) -> PacketState;
    fn bound() -> PacketDirection;
}

pub struct Pipeline<'a> {
    encoder_pipeline: Vec<(Option<&'a str>, Box<dyn Writable>)>,
    decoder_pipeline: Vec<(Option<&'a str>, Box<dyn Writable>)>
}

impl<'a> Pipeline<'a> {

    pub fn add_last_encoder(mut self, encoder: impl Writable + 'static, name: Option<&'a str>) -> Self {
        self.encoder_pipeline.push((name, Box::new(encoder)));
        self
    }

    pub fn add_last_decoder(mut self, decoder: impl Writable + 'static, name: Option<&'a str>) -> Self {
        self.decoder_pipeline.push((name, Box::new(decoder)));
        self
    }

    pub fn encode(&self, buffer: Buffer) -> Result<Buffer, Error> {
        let mut cloned_buffer = buffer.clone();
        for (_, encoder) in &self.encoder_pipeline {
            cloned_buffer = encoder.write(cloned_buffer)?;
        }
        Ok(cloned_buffer)
    }

    pub fn decode(&self, buffer: Buffer) -> Result<Buffer, Error> {
        let mut cloned_buffer = buffer.clone();
        for (_, decoder) in &self.decoder_pipeline {
            cloned_buffer = decoder.write(cloned_buffer)?;
        }
        Ok(cloned_buffer)
    }
}

impl<'a> Pipeline<'a> {
    pub fn new() -> Self {
        Self { encoder_pipeline: Vec::new(), decoder_pipeline: Vec::new() }
    }
}

pub trait Writable {
    fn write(&self, buffer: Buffer) -> Result<Buffer, Error>;
}

pub trait Readable {
    fn read(buffer: Buffer) -> Result<Self, Error> where Self: Sized;
}

macro_rules! define_type_io {
    ($_type: tt) => {
        paste::paste! {
            impl Writable for $_type {
                fn write(&self, mut buffer: Buffer) -> Result<Buffer, Error> {
                    buffer.[<write_ $_type>](*self)?;
                    Ok(buffer)
                }
            }

            impl Readable for $_type {
                fn read(mut buffer: Buffer) -> Result<Self, Error> {
                    Ok(buffer.[<read_ $_type>]()?)
                }
            }
        }
    }
}

macro_rules! define_var_int {
    ($_type: tt) => {
        paste::paste! {
            pub struct [<Var $_type:upper>] {
                pub value: $_type
            }

            impl Writable for [<Var $_type:upper>] {
                fn write(&self, mut buffer: Buffer) -> Result<Buffer, Error> {
                    buffer.[<write_var_ $_type>](self.value)?;
                    Ok(buffer)
                }
            }

            impl Readable for [<Var $_type:upper>] {
                fn read(mut buffer: Buffer) -> Result<Self, Error> {
                    Ok(Self { value: buffer.[<read_var_ $_type>]()? })
                }
            }
        }
    }
}

define_var_int!(i32);

define_type_io!(u8);
define_type_io!(u16);
define_type_io!(u32);
define_type_io!(u64);

define_type_io!(i8);
define_type_io!(i16);
define_type_io!(i32);
define_type_io!(i64);

impl Writable for Uuid {
    fn write(&self, mut buffer: Buffer) -> Result<Buffer, Error> {
        let bits = self.as_u64_pair();
        buffer.write_u64(bits.0)?;
        buffer.write_u64(bits.1)?;
        Ok(buffer.clone())
    }
}

impl Readable for Uuid {
    fn read(mut buffer: Buffer) -> Result<Self, Error> where Self: Sized {
        let most_significant_bits = buffer.read_u64()?;
        let least_significant_bits = buffer.read_u64()?;
        Ok(Uuid::from_u64_pair(most_significant_bits, least_significant_bits))
    }
}

impl<T: Writable> Writable for Vec<T> {
    fn write(&self, mut buffer: Buffer) -> Result<Buffer, Error> {
        buffer.write_var_i32(self.len() as i32)?;
        for element in self {
            buffer = element.write(buffer)?;
        }
        Ok(buffer.clone())
    }
}

impl<T: Readable> Readable for Vec<T> {
    fn read(mut buffer: Buffer) -> Result<Self, Error> where Self: Sized {
        let length = buffer.read_var_i32()?;
        if length < 0 {
            return Err(Error::other("Unable to read array with negative length!".to_string()));
        }

        let mut vector: Vec<T> = Vec::new();
        for _ in 0..length {
            vector.push(T::read(buffer.clone())?);
            buffer.set_position(buffer.position() + size_of::<T>());
        }
        Ok(vector)
    }
}

impl Writable for String {
    fn write(&self, mut buffer: Buffer) -> Result<Buffer, Error> {
        buffer.write_string(self.clone())?;
        Ok(buffer)
    }
}

impl Readable for String {
    fn read(mut buffer: Buffer) -> Result<Self, Error> where Self: Sized {
        buffer.read_string()
    }
}