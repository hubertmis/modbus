use crate::Error;
use crate::pdu::{Function, FunctionCode, Request, Response};
use std::convert::{Infallible, TryFrom, TryInto};

#[derive(Clone, Copy, Debug, FromPrimitive, PartialEq)]
enum Value {
    Off = 0x0000,
    On  = 0xFF00,
}

impl TryFrom<[u8; 2]> for Value {
    type Error = Error;

    fn try_from(value: [u8; 2]) -> Result<Self, Self::Error> {
        match u16::from_be_bytes(value) {
            x if x == Value::Off as u16 => Ok(Value::Off),
            x if x == Value::On as u16 => Ok(Value::On),
            _ => Err(Error::InvalidData),
        }
    }
}

impl TryFrom<&[u8]> for Value {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 2 {
            return Err(Error::InvalidDataLength);
        }
        let val_array: [u8; 2] = value.try_into().unwrap();

        Value::try_from(val_array)
    }
}

impl TryFrom<bool> for Value {
    type Error = Infallible;

    fn try_from(value: bool) -> Result<Self, Self::Error> {
        match value {
            true => Ok(Value::On),
            false => Ok(Value::Off),
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = Infallible;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::On => Ok(true),
            Value::Off => Ok(false),
        }
    }
}

/// Write Single Coil request or response function
#[derive(Debug, PartialEq)]
pub struct Message {
    address: u16,
    value: Value,
}

impl Message {
    /// Create a new Write Single Coil function
    /// 
    /// # Examples
    /// ```
    /// let req = modbus::WriteSingleCoilRequest::new(0xabcd, true);
    /// let rsp = modbus::WriteSingleCoilResponse::new(0x0123, false);
    /// ```
    pub fn new(address: u16, value: bool) -> Self {
        Message{address, value: value.try_into().unwrap()}
    }

    /// Get address of the coil from the Write Single Coil function
    /// 
    /// # Examples
    /// ```
    /// let address = 0x0abc;
    /// let rsp = modbus::WriteSingleCoilResponse::new(address, false);
    /// assert_eq!(rsp.get_address(), address);
    /// ```
    pub fn get_address(&self) -> u16 {
        self.address
    }

    /// Get value from the Write Single Coil function
    /// 
    /// # Examples
    /// ```
    /// let value = true;
    /// let req = modbus::WriteSingleCoilRequest::new(0xfedc, value);
    /// assert_eq!(req.get_value(), value);
    /// ```
    pub fn get_value(&self) -> bool {
        self.value.try_into().unwrap()
    }
}

impl Function for Message {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut result = Vec::new();
        result.push(FunctionCode::WriteSingleCoil as u8);
        result.append(&mut self.address.to_be_bytes().to_vec());
        result.append(&mut (self.value as u16).to_be_bytes().to_vec());

        Ok(result)
    }

    fn decode(data: &[u8]) -> Result<Self, Error> {
        if data.len() != 5 {
            return Err(Error::InvalidDataLength);
        }
        if data[0] != FunctionCode::WriteSingleCoil as u8 {
            return Err(Error::InvalidData);
        }
        
        Ok(Self{address: u16::from_be_bytes(data[1..=2].try_into().unwrap()),
                value: data[3..=4].try_into()?})
    }
}

impl Request for Message {
    type Rsp = Message;
}

impl Response for Message {
    fn get_exc_function_code() -> u8 {
        FunctionCode::ExcWriteSingleCoil.try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_request() {
        let req = Message::new(0xdead, true);
        let pdu = req.encode().unwrap();
        let expected_pdu = vec![0x05, 0xde, 0xad, 0xff, 0x00];

        assert_eq!(pdu, expected_pdu);
    }

    #[test]
    fn test_encode_response() {
        let rsp = Message::new(0xffff, false);
        let pdu = rsp.encode().unwrap();
        let expected_pdu = vec![0x05, 0xff, 0xff, 0x00, 0x00];

        assert_eq!(pdu, expected_pdu);
    }

    #[test]
    fn test_decode_request() {
        let pdu = vec![0x05, 0x00, 0x00, 0xff, 0x00];
        let req = Message::decode(&pdu).unwrap();
        let expected_req = Message::new(0x0000, true);

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
        let pdu = vec![0x05, 0x01, 0x23, 0xff, 0x00];
        let rsp = Message::decode(&pdu).unwrap();
        let expected_rsp = Message::new(0x0123, true);

        assert_eq!(rsp, expected_rsp);
    }

}