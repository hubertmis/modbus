use crate::error::Error;
use crate::pdu::{Function, FunctionCode, MAX_SIZE, Request as ReqT, Response as RspT};
use super::DSCR_PER_BYTE;
use std::convert::TryInto;
use std::vec::Vec;

/// Read Coils function request
#[derive(Debug, PartialEq)]
pub struct Request {
    address: u16,
    quantity: u16,
}

impl Request {
    /// Create a new Read Coils request
    /// 
    /// # Examples
    /// 
    /// ```
    /// let request = modbus::ReadCoilsRequest::new(0x000a, 0x0004);
    /// ```
    pub fn new(address: u16, quantity: u16) -> Self {
        // TODO: debug_assert quantity > 0
        Request{address, quantity}
    }

    /// Get address of the first coil from the request
    /// 
    /// # Examples
    /// 
    /// ```
    /// let address = 0x01234;
    /// let request = modbus::ReadCoilsRequest::new(address, 0x0001);
    /// 
    /// assert_eq!(request.get_address(), address);
    /// ```
    pub fn get_address(&self) -> u16 {
        self.address
    }

    /// Get quantity of the coils in the request
    /// 
    /// # Examples
    /// 
    /// ```
    /// let quantity = 35;
    /// let request = modbus::ReadCoilsRequest::new(0, quantity);
    /// 
    /// assert_eq!(request.get_quantity(), quantity);
    /// ```
    pub fn get_quantity(&self) -> u16 {
        self.quantity
    }
}

impl Function for Request {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        match self.quantity {
            1..=2000 => {
                let mut result = Vec::new();
                result.push(FunctionCode::ReadCoils as u8);
                result.append(&mut self.address.to_be_bytes().to_vec());
                result.append(&mut self.quantity.to_be_bytes().to_vec());

                Ok(result)
            }
            _ => Err(Error::InvalidValue),
        }
    }

    fn decode(data: &[u8]) -> Result<Self, Error> where Self: Sized {
        if data.len() != 5 {
            return Err(Error::InvalidDataLength);
        }
        if data[0] != FunctionCode::ReadCoils as u8 {
            return Err(Error::InvalidData);
        }

        Ok(Self {address: u16::from_be_bytes(data[1..=2].try_into().unwrap()), 
                 quantity: u16::from_be_bytes(data[3..=4].try_into().unwrap())})
    }
}

impl ReqT for Request {
    type Rsp = Response;
}

/// Read Coils function response
#[derive(Debug, PartialEq)]
pub struct Response {
    coils: Vec<bool>,
}

impl Response {
    /// Create a new Read Coils response.
    /// 
    /// # Examples
    /// ```
    /// let response = modbus::ReadCoilsResponse::new(&[true, false]);
    /// ```
    pub fn new(coils: &[bool]) -> Self {
        Self {coils: coils.to_vec()}
    }

    /// Get vector of coils from the given response.
    /// 
    /// # Examples
    /// ```
    /// let coil_values = [true, false, true, true, false, false, true, false];
    /// let response = modbus::ReadCoilsResponse::new(&coil_values);
    /// let new_coil_values = response.get_coils();
    /// assert_eq!(&coil_values.to_vec(), new_coil_values)
    /// ```
    pub fn get_coils(&self) -> &Vec<bool> {
        &self.coils
    }
}

impl Function for Response {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        const MAX_BYTE_COUNT: usize = MAX_SIZE - 2;
        let byte_count = self.coils.len() / DSCR_PER_BYTE + if self.coils.len() % DSCR_PER_BYTE > 0 { 1 } else { 0 };

        match byte_count {
            0 => Err(Error::InvalidValue),
            1..=MAX_BYTE_COUNT => {
                let mut result = Vec::new();
                result.push(FunctionCode::ReadCoils as u8);
                result.push(byte_count as u8);

                for byte_num in 0..byte_count {
                    let mut byte: u8 = 0;

                    for bit_num in 0..DSCR_PER_BYTE {
                        let coil_id = byte_num * DSCR_PER_BYTE + bit_num;
                        if coil_id >= self.coils.len() {
                            break;
                        }

                        if self.coils[coil_id] {
                            byte |= 1 << bit_num;
                        }
                    }

                    result.push(byte);
                }

                Ok(result)
            }
            _ => Err(Error::InvalidValue),
        }
    }

    fn decode(data: &[u8]) -> Result<Self, Error> where Self: Sized {
        if data.len() < 3 {
            return Err(Error::InvalidDataLength);
        }
        if data[0] != FunctionCode::ReadCoils as u8 {
            return Err(Error::InvalidData);
        }

        let byte_count = data[1] as usize;
        if data.len() != byte_count + 2 {
            return Err(Error::InvalidDataLength);
        }

        let mut result = Vec::new();
        for byte_num in 0..byte_count {
            for bit_num in 0..DSCR_PER_BYTE {
                result.push(if data[2 + byte_num] & (1 << bit_num) != 0 { true } else { false });
            }
        }

        Ok(Self {coils: result})
    }
}

impl RspT for Response {
    fn get_exc_function_code() -> u8 {
        FunctionCode::ExcReadCoils.try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_read_coils_request() {
        let pdu = Request{address: 0x1234, quantity: 0x00cd}.encode().unwrap();
        let expected_pdu = vec![0x01, 0x12, 0x34, 0x00, 0xcd];
        assert_eq!(pdu, expected_pdu);
    }

    #[test]
    fn test_encode_read_zero_coils_request() {
        let result = Request{address: 0x1234, quantity: 0}.encode().err().unwrap();
        match result {
            Error::InvalidValue => {}
            _ => panic!(format!("Expected InvalidValue, but got {:?}", result)),
        }
    }

    #[test]
    fn test_encode_read_coils_response() {
        let pdu = Response{coils: vec![true, false, true, true, false, false, true, true,
                                       true, true, false, true, false, true, true, false,
                                       true, false, true]}.encode().unwrap();
        let expected_pdu = vec![0x01, 0x03, 0xCD, 0x6B, 0x05];
        assert_eq!(pdu, expected_pdu);
    }

    #[test]
    fn test_encode_read_zero_coils_response() {
        let result = Response{coils: vec![]}.encode().err().unwrap();
        match result {
            Error::InvalidValue => {}
            _ => panic!(format!("Expected InvalidValue, but got {:?}", result)),
        }
    }

    #[test]
    fn test_decode_read_coils_request() {
        let pdu = [0x01, 0x12, 0x34, 0xab, 0xcd];
        let result = Request::decode(&pdu).unwrap();
        assert_eq!(result.address, 0x1234);
        assert_eq!(result.quantity, 0xabcd);
    }

    #[test]
    fn test_decode_read_coils_response() {
        let pdu = [0x01, 0x03, 0xCD, 0x6B, 0x05];
        let result = Response::decode(&pdu).unwrap();
        for (i, expected_value) in [true, false, true, true, false, false, true, true,
                                    true, true, false, true, false, true, true, false,
                                    true, false, true].iter().enumerate() {
            assert_eq!(result.coils[i], *expected_value);
        }
    }
}
