use crate::Error;
use crate::pdu::{Function, FunctionCode, Request, Response, Setter};
use std::convert::TryInto;

/// Write Single Register request or response function
#[derive(Debug, PartialEq)]
pub struct Message {
    address: u16,
    value: u16,
}

impl Message {
    /// Create a new Write Single Register function
    /// 
    /// # Examples
    /// ```
    /// let req = modbus::WriteSingleRegRequest::new(0xabcd, 0xcafe);
    /// let rsp = modbus::WriteSingleRegResponse::new(0x0123, 0xface);
    /// ```
    pub fn new(address: u16, value: u16) -> Self {
        Message{address, value}
    }

    /// Get address of the register from the Write Single Reigster function
    /// 
    /// # Examples
    /// ```
    /// let address = 0x0abc;
    /// let rsp = modbus::WriteSingleRegResponse::new(address, 0x0000);
    /// assert_eq!(rsp.get_address(), address);
    /// ```
    pub fn get_address(&self) -> u16 {
        self.address
    }

    /// Get value from the Write Single Register function
    /// 
    /// # Examples
    /// ```
    /// let value = 0x0123;
    /// let req = modbus::WriteSingleRegRequest::new(0xfedc, value);
    /// assert_eq!(req.get_value(), value);
    /// ```
    pub fn get_value(&self) -> u16 {
        self.value
    }
}

impl Function for Message {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut result = Vec::new();
        result.push(FunctionCode::WriteSingleReg as u8);
        result.append(&mut self.address.to_be_bytes().to_vec());
        result.append(&mut self.value.to_be_bytes().to_vec());

        Ok(result)
    }

    fn decode(data: &[u8]) -> Result<Self, Error> {
        if data.len() != 5 {
            return Err(Error::InvalidDataLength);
        }
        if data[0] != FunctionCode::WriteSingleReg as u8 {
            return Err(Error::InvalidData);
        }
        
        Ok(Self{address: u16::from_be_bytes(data[1..=2].try_into().unwrap()),
                value: u16::from_be_bytes(data[3..=4].try_into().unwrap())})
    }
}

impl Request for Message {
    type Rsp = Message;
}

impl Response for Message {
    fn get_exc_function_code() -> u8 {
        FunctionCode::ExcWriteSingleReg.try_into().unwrap()
    }
}

impl Setter for Message {

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_request() {
        let req = Message::new(0xdead, 0xfade);
        let pdu = req.encode().unwrap();
        let expected_pdu = vec![0x06, 0xde, 0xad, 0xfa, 0xde];

        assert_eq!(pdu, expected_pdu);
    }

    #[test]
    fn test_encode_response() {
        let rsp = Message::new(0xffff, 0x0102);
        let pdu = rsp.encode().unwrap();
        let expected_pdu = vec![0x06, 0xff, 0xff, 0x01, 0x02];

        assert_eq!(pdu, expected_pdu);
    }

    #[test]
    fn test_decode_request() {
        let pdu = vec![0x06, 0x00, 0x00, 0xff, 0x00];
        let req = Message::decode(&pdu).unwrap();
        let expected_req = Message::new(0x0000, 0xff00);

        assert_eq!(req, expected_req);
    }

    #[test]
    fn test_decode_invalid_request() {
        let pdu = vec![0x05, 0x01, 0x23, 0x00, 0x01];
        let err = Message::decode(&pdu).err().unwrap();
        match err {
            Error::InvalidData => {}
            _ => panic!(format!("Expected InvalidData, but got {:?}", err)),
        }
    }

    #[test]
    fn test_decode_response() {
        let pdu = vec![0x06, 0x01, 0x23, 0x87, 0x65];
        let rsp = Message::decode(&pdu).unwrap();
        let expected_rsp = Message::new(0x0123, 0x8765);

        assert_eq!(rsp, expected_rsp);
    }
}
