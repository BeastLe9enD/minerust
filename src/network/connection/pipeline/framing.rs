use crate::network::buffer::Buffer;
use crate::network::connection::Writable;
use crate::network::Error;

pub struct FrameEncoder {

}

impl Writable for FrameEncoder {
    fn write(&self, buffer: Buffer) -> Result<Buffer, Error> {
        let buffer_bytes = buffer.to_bytes();

        let mut copied_buffer = buffer.cloned_metadata();
        copied_buffer.write_var_i32(buffer_bytes.len() as i32)?;
        copied_buffer.write_all(buffer_bytes);

        Ok(copied_buffer)
    }
}

impl FrameEncoder {
    pub fn new() -> Self {
        Self {}
    }
}