use std::convert::TryInto;
use std::vec::Vec;

use crate::Error;
use crate::pdu::{MAX_SIZE, Function, Request as ReqT, Response as RspT, FunctionCode};
use super::DSCR_PER_BYTE;

/// Read Discrete Inputs function request
#[derive(Debug, PartialEq)]
pub struct Request {
    address: u16,
    quantity: u16,
}

impl Request {
    /// Create a new Read Discrete Inputs request
    /// 
    /// # Examples
    /// 
    /// ```
    /// let request = modbus::ReadDscrInRequest::new(0x000a, 0x0004);
    /// ```
    pub fn new(address: u16, quantity: u16) -> Self {
        Request{address, quantity}
    }

    /// Get address of the first discrete input from the request
    /// 
    /// # Examples
    /// 
    /// ```
    /// let address = 0x01234;
    /// let request = modbus::ReadDscrInRequest::new(address, 0x0001);
    /// 
    /// assert_eq!(request.get_address(), address);
    /// ```
    pub fn get_address(&self) -> u16 {
        self.address
    }

    /// Get quantity of the discrete inputs in the request
    /// 
    /// # Examples
    /// 
    /// ```
    /// let quantity = 35;
    /// let request = modbus::ReadDscrInRequest::new(0, quantity);
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
                result.push(FunctionCode::ReadDscrIn as u8);
                result.append(&mut self.address.to_be_bytes().to_vec());
                result.append(&mut self.quantity.to_be_bytes().to_vec());
                Ok(result)
            }
            _ => Err(Error::InvalidValue)
        }
    }

    fn decode(data: &[u8]) -> Result<Self, Error> {
        if data.len() != 5 {
            return Err(Error::InvalidDataLength);
        }
        if data[0] != FunctionCode::ReadDscrIn as u8 {
            return Err(Error::InvalidData);
        }

        Ok(Self {address: u16::from_be_bytes(data[1..=2].try_into().unwrap()),
                 quantity: u16::from_be_bytes(data[3..=4].try_into().unwrap())})
    }
}

impl ReqT for Request {
    type Rsp = Response;
}

/// Read Discrete Inputs function response
#[derive(Debug, PartialEq)]
pub struct Response {
    inputs: Vec<bool>,
}

impl Response {
    /// Create a new Read Discrete Inputs response
    /// 
    /// # Examples
    /// ```
    /// let response = modbus::ReadDscrInResponse::new(&[false, true, false]);
    /// ```
    pub fn new(inputs: &[bool]) -> Self {
        Self {inputs: inputs.to_vec()}
    }

    /// Get list of inputs from the Read Discrete Inputs response
    /// 
    /// # Examples
    /// ```
    /// let inputs = vec![true, true, false, false];
    /// let response = modbus::ReadDscrInResponse::new(&inputs);
    /// assert_eq!(response.get_inputs(), &inputs);
    /// ```
    pub fn get_inputs(&self) -> &Vec<bool> {
        &self.inputs
    }
}

impl Function for Response {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        let in_cnt = self.inputs.len();
        let byte_count = in_cnt / DSCR_PER_BYTE + if in_cnt % DSCR_PER_BYTE != 0 { 1 } else { 0 };
        const MAX_BYTE_COUNT: usize = MAX_SIZE - 3;
        
        match byte_count {
            1..=MAX_BYTE_COUNT => {
                let mut result = Vec::new();
                result.push(FunctionCode::ReadDscrIn as u8);
                result.push(byte_count as u8);

                for byte_num in 0..byte_count {
                    let mut byte: u8 = 0;
                    for bit_num in 0..DSCR_PER_BYTE {
                        let i = byte_num * DSCR_PER_BYTE + bit_num;
                        if i >= self.inputs.len() {
                            break;
                        }

                        if self.inputs[i] {
                            byte |= 1 << bit_num;
                        }
                    }

                    result.push(byte);
                }

                Ok(result)
            }
            _ => Err(Error::InvalidValue)
        }
    }

    fn decode(data: &[u8]) -> Result<Self, Error> {
        if data.len() < 2 {
            return Err(Error::InvalidDataLength);
        }
        if data[0] != FunctionCode::ReadDscrIn as u8 {
            return Err(Error::InvalidData);
        }

        let byte_count = data[1] as usize;
        if data.len() != byte_count + 2 {
            return Err(Error::InvalidDataLength);
        }

        let mut result = Self{inputs: Vec::with_capacity(byte_count * DSCR_PER_BYTE)};
        for byte_num in 2..2+byte_count {
            for bit_num in 0..DSCR_PER_BYTE {
                result.inputs.push(if data[byte_num] & (1 << bit_num) != 0 { true } else { false });
            }
        }

        Ok(result)
    }
}

impl RspT for Response {
    fn get_exc_function_code() -> u8 {
        FunctionCode::ExcReadDscrIn.try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_req() {
        let req = Request::new(0x1234, 0x7D0);
        let pdu = req.encode().unwrap();
        let expected_pdu = vec![0x02, 0x12, 0x34, 0x07, 0xD0];

        assert_eq!(pdu, expected_pdu);
    }

    #[test]
    fn decode_req() {
        let pdu = vec![0x02, 0xab, 0xcd, 0x01, 0x23];
        let req = Request::decode(&pdu).unwrap();
        let expected_req = Request {address: 0xabcd, quantity: 0x0123};

        assert_eq!(req, expected_req);
    }

    #[test]
    fn encode_rsp() {
        let rsp = Response{inputs: vec![false, false, true, true, false, true, false, true,
                                        true, true, false, true, true, false, true, true,
                                        true, false, true, false, true, true]};
        let pdu = rsp.encode().unwrap();
        let expected_pdu = vec![0x02, 0x03, 0xAC, 0xDB, 0x35];

        assert_eq!(pdu, expected_pdu);
    }

    #[test]
    fn decode_rsp() {
        let pdu = vec![0x02, 0x03, 0xAC, 0xDB, 0x35];
        let rsp = Response::decode(&pdu).unwrap();
        let expected_rsp = Response{inputs: vec![false, false, true, true, false, true, false, true,
                                                 true, true, false, true, true, false, true, true,
                                                 true, false, true, false, true, true, false, false]};

        assert_eq!(rsp, expected_rsp);
    }
}