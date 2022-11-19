use std::mem::size_of;
use crate::network::{ByteOrder, Error};

const LAST_SEVEN_BITS: i32 = 0b01111111;
const NEXT_BYTE_EXISTS: u8 = 0b10000000;
const SEVEN_BITS_SHIFT_MASK: i32 = 0x01ffffff;

macro_rules! var_int {
    ($_type: tt, $read_length: expr) => {
        paste::paste! {
            pub fn [<write_var_ $_type>](&mut self, mut value: $_type) -> Result<(), Error> {
                loop {
                    let mut temp = (value & LAST_SEVEN_BITS) as u8;
                    value >>= 7;
                    value &= SEVEN_BITS_SHIFT_MASK;
                    if value != 0 {
                        temp |= NEXT_BYTE_EXISTS;
                    }

                    self.write_u8(temp)?;
                    if value == 0 {
                        break;
                    }
                }
                Ok(())
            }

            pub fn [<read_var_ $_type>](&mut self) -> Result<$_type, Error> {
                let mut value = 0;
                for i in 0..$read_length {
                    let read = self.read_u8()?;
                    value |= ((read & 0b0111_1111) as $_type) << 7 * i;
                    if value & 0b1000_0000 == 0 {
                        break;
                    }
                }
                Ok(value)
            }
        }
    }
}

macro_rules! buffer_method {
    ($_type: tt) => {
        paste::paste! {
            pub fn [<write_ $_type>](&mut self, value: $_type) -> Result<(), Error> {
                let bytes = if (if let Some(byte_order) = self.byte_order() { byte_order } else { ByteOrder::system_order() }) == ByteOrder::LittleEndian {
                    value.to_le_bytes()
                } else {
                    value.to_be_bytes()
                };

                for byte in bytes {
                    self.write_u8(byte)?;
                }
                Ok(())
            }

            pub fn [<read_ $_type>](&mut self) -> Result<$_type, Error> {
                let mut array: [u8; size_of::<$_type>()] = Default::default();
                for i in 0..size_of::<$_type>() {
                    array[i] = self.read_u8()?;
                }

                Ok(if (if let Some(byte_order) = self.byte_order() { byte_order } else { ByteOrder::system_order() }) == ByteOrder::LittleEndian {
                    $_type::from_le_bytes(array)
                } else {
                    $_type::from_be_bytes(array)
                })
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Buffer {
    data: Vec<u8>,
    writable: bool,
    order: Option<ByteOrder>,
    position: usize
}

impl Buffer {
    pub fn new(data: Vec<u8>, writable: bool, order: Option<ByteOrder>) -> Self {
        Self { data, writable, order, position: 0 }
    }

    pub fn empty(writable: bool, order: Option<ByteOrder>) -> Self {
        Self::new(Vec::new(), writable, order)
    }

    pub fn cloned_metadata(&self) -> Self {
        Self { data: Vec::new(), writable: self.writable, order: self.order.clone(), position: 0 }
    }

    pub fn write_all(&mut self, bytes: Vec<u8>) {
        for byte in bytes {
            self.data.push(byte);
        }
    }

    pub fn write_u8(&mut self, value: u8) -> Result<(), Error> {
        if !self.writable() {
            return Err(Error::not_writable("Buffer"));
        }

        self.data.push(value);
        self.position += 1;
        Ok(())
    }

    pub fn read_u8(&mut self) -> Result<u8, Error> {
        self.position += 1;
        Ok(self.data[self.position - 1])
    }

    pub fn write_str(&mut self, string: &str) -> Result<(), Error> {
        self.write_string(String::from(string))
    }

    pub fn write_string(&mut self, string: String) -> Result<(), Error> {
        self.write_var_i32(string.len() as i32)?;
        for byte in string.bytes() {
            self.write_u8(byte)?;
        }
        Ok(())
    }

    pub fn read_string(&mut self) -> Result<String, Error> {
        let mut bytes = Vec::new();
        for _ in 0..self.read_var_i32()? {
            bytes.push(self.read_u8()?);
        }
        Ok(unsafe { String::from_utf8_unchecked(bytes) })
    }

    var_int!(i32, 4);

    buffer_method!(u16);
    buffer_method!(u32);
    buffer_method!(u64);

    buffer_method!(i8);
    buffer_method!(i16);
    buffer_method!(i32);
    buffer_method!(i64);

    pub fn set_position(&mut self, position: usize) {
        self.position = position;
    }

    pub fn byte_order(&self) -> Option<ByteOrder> {
        self.order.clone()
    }

    pub fn writable(&self) -> bool {
        self.writable
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn reset(&mut self) {
        self.position = 0;
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.position = 0;
    }
}