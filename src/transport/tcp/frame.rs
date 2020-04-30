use crate::error::Error;
use std::convert::TryInto;
use std::sync::atomic::{AtomicU16, Ordering};

const MODBUS_ID: u16 = 0;
static TRANSACTION_ID: AtomicU16 = AtomicU16::new(0);

fn get_transaction_id() -> u16 {
    TRANSACTION_ID.fetch_add(1, Ordering::Relaxed)
}

pub struct Frame<'a> {
    transaction_id: u16,
    unit_id: u8,
    pdu: &'a [u8],
}

impl<'a> Frame<'a> {
    pub fn new(unit_id: u8, pdu: &'a [u8]) -> Self {
        Self{transaction_id: get_transaction_id(), unit_id, pdu}
    }

    pub fn get_unit_id(&self) -> u8 {
        self.unit_id
    }

    pub fn get_pdu(&self) -> Vec<u8> {
        self.pdu.to_vec()
    }

    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut result = Vec::new();
        result.append(&mut self.transaction_id.to_be_bytes().to_vec());
        result.append(&mut MODBUS_ID.to_be_bytes().to_vec());
        result.append(&mut ((self.pdu.len() + 1) as u16).to_be_bytes().to_vec());
        result.push(self.unit_id);
        result.append(&mut self.pdu.to_vec());

        Ok(result)
    }

    pub fn decode(data: &'a [u8]) -> Result<Self, Error> {
        let len = data.len();
        if len < 8 {
            return Err(Error::TooShortData);
        }
        if u16::from_be_bytes(data[2..=3].try_into().unwrap()) != MODBUS_ID {
            return Err(Error::InvalidData);
        }

        let expected_len = (u16::from_be_bytes(data[4..=5].try_into().unwrap()) + 6) as usize;
        if len < expected_len {
            return Err(Error::TooShortData);
        }
        if len > expected_len {
            return Err(Error::InvalidDataLength);
        }

        Ok(Self{transaction_id: u16::from_be_bytes(data[0..=1].try_into().unwrap()), 
                unit_id: data[6],
                pdu: &data[7..]})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let mut frame = Frame::new(0xFF, &[0x03, 0x00, 0x04, 0x00, 0x01]);
        frame.transaction_id = 0x1501;
        let frame = frame.encode().unwrap();
        let expected_frame = vec![0x15, 0x01, 0x00, 0x00, 0x00, 0x06, 0xFF, 0x03, 0x00, 0x04, 0x00, 0x01];
        assert_eq!(frame, expected_frame);
    }

    #[test]
    fn test_decode() {
        let frame_data = vec![0x15, 0x01, 0x00, 0x00, 0x00, 0x06, 0xFF, 0x03, 0x00, 0x04, 0x00, 0x01];
        let frame = Frame::decode(&frame_data).unwrap();

        assert_eq!(frame.transaction_id, 0x1501);
        assert_eq!(frame.unit_id, 0xFF);
        assert_eq!(frame.pdu, &frame_data[7..]);
    }
}