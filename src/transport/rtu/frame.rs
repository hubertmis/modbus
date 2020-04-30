use std::convert::TryInto;
use std::vec::Vec;

use crc16;

use crate::error::Error;

pub struct Frame<'a> {
    address: u8,
    pdu: &'a [u8],
}

impl<'a> Frame<'a> {
    pub fn new(address: u8, pdu: &'a [u8]) -> Self {
        Frame{address, pdu}
    }

    pub fn get_pdu(&self) -> Vec<u8> {
        Vec::from(self.pdu)
    }

    pub fn is_address(&self, other: u8) -> bool {
        self.address == other
    }

    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut result = Vec::new();
        result.push(self.address);
        result.append(&mut self.pdu.to_vec());

        let crc = crc16::State::<crc16::MODBUS>::calculate(&result);
        result.append(&mut crc.to_le_bytes().to_vec());

        Ok(result)
    }

    pub fn decode(data: &'a [u8]) -> Result<Self, Error> {
        let len = data.len();
        if len < 4 {
            return Err(Error::InvalidDataLength);
        }

        let expected_crc = crc16::State::<crc16::MODBUS>::calculate(&data[0..len-2]);
        let crc = u16::from_le_bytes(data[len-2..len].try_into().unwrap());

        if expected_crc != crc {
            return Err(Error::InvalidData);
        }

        Ok(Self{address: data[0], pdu: &data[1..len-2]})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let frame = Frame::new(2, &[0x07]).encode().unwrap();
        let expected_frame = vec![0x02, 0x07, 0x41, 0x12];
        assert_eq!(frame, expected_frame);
    }

    #[test]
    fn test_decode() {
        let frame_data = vec![0x02, 0x07, 0x41, 0x12];
        let frame = Frame::decode(&frame_data).unwrap();

        assert_eq!(frame.address, 2);
        assert_eq!(frame.pdu, &frame_data[1..=1]);
    }

    #[test]
    fn test_decode_invalid_crc() {
        let frame_data = [0x02, 0x07, 0x41, 0x00];
        let err = Frame::decode(&frame_data).err().unwrap();

        match err {
            Error::InvalidData => {}
            _ => panic!(format!("Expected InvalidData, but got {:?}", err)),
        }
    }
}