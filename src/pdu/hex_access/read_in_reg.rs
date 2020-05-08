use crate::error::Error;
use crate::pdu::{Function, FunctionCode, Request as ReqT, Response as RspT};
use std::convert::TryInto;
use std::vec::Vec;

const MIN_QUANTITY: u16 = 1;
const MAX_QUANTITY: u16 = 0x7D;

/// Read Input Registers function request
#[derive(Debug, PartialEq)]
pub struct Request {
    address: u16,
    quantity: u16,
}

impl Request {
    /// Create a new Read Input registers request
    /// 
    /// # Examples
    /// ```
    /// let req = modbus::ReadInRegRequest::new(0x0102, 0x0001);
    /// ```
    pub fn new(address: u16, quantity: u16) -> Self {
        Self {address, quantity}
    }

    /// Get address of the first register from the request
    /// 
    /// # Examples
    /// 
    /// ```
    /// let address = 0x4321;
    /// let request = modbus::ReadInRegRequest::new(address, 0x0001);
    /// 
    /// assert_eq!(request.get_address(), address);
    /// ```
    pub fn get_address(&self) -> u16 {
        self.address
    }

    /// Get quantity of the registers in the request
    /// 
    /// # Examples
    /// 
    /// ```
    /// let quantity = 125;
    /// let request = modbus::ReadInRegRequest::new(0, quantity);
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
            MIN_QUANTITY..=MAX_QUANTITY => {
                let mut result = Vec::new();
                result.push(FunctionCode::ReadInReg as u8);
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
        if data[0] != FunctionCode::ReadInReg as u8 {
            return Err(Error::InvalidData);
        }

        Ok(Self {address: u16::from_be_bytes(data[1..=2].try_into().unwrap()), 
                 quantity: u16::from_be_bytes(data[3..=4].try_into().unwrap())})
    }
}

impl ReqT for Request {
    type Rsp = Response;
}

/// Read Holding Registers function response
pub struct Response {
    registers: Vec<u16>,
}

impl Response {
    /// Create a new Read Holding Registers response
    /// 
    /// # Examples
    /// ```
    /// let registers: [u16; 1] = [0x1023];
    /// let rsp = modbus::ReadInRegResponse::new(&registers);
    /// ```
    pub fn new(registers: &[u16]) -> Self {
        Self{ registers: registers.to_vec() }
    }

    /// Get registers' values from the response.
    /// 
    /// # Examples
    /// ```
    /// let registers = vec![0x2047, 0x0000, 0x0123];
    /// let rsp = modbus::ReadInRegResponse::new(&registers);
    /// assert_eq!(rsp.get_registers(), &registers);
    /// ```
    pub fn get_registers(&self) -> &Vec<u16> {
        &self.registers
    }
}

impl Function for Response {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut result = Vec::new();
        result.push(FunctionCode::ReadInReg as u8);
        result.push((self.registers.len() * 2) as u8);
        for reg in &self.registers {
            result.append(&mut reg.to_be_bytes().to_vec());
        }

        Ok(result)
    }

    fn decode(data: &[u8]) -> Result<Self, Error> where Self: Sized {
        if data.len() < 2 {
            return Err(Error::InvalidDataLength);
        }
        if data[0] != FunctionCode::ReadInReg as u8 {
            return Err(Error::InvalidData);
        }

        let num_bytes = data[1];
        if num_bytes % 2 != 0 {
            return Err(Error::InvalidData);
        }
        if num_bytes as usize != data.len() - 2 {
            return Err(Error::InvalidDataLength);
        }

        let num_registers = (num_bytes / 2) as usize;
        let mut registers = Vec::with_capacity(num_registers);
        for i in 0..num_registers {
            let reg_idx = 2 + 2 * i;
            let reg = u16::from_be_bytes(data[reg_idx..=(reg_idx+1)].try_into().unwrap());
            registers.push(reg);
        }
        Ok(Self {registers})
    }
}

impl RspT for Response {
    fn get_exc_function_code() -> u8 {
        FunctionCode::ExcReadInReg.try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_request() {
        let req = Request::new(0x0102, 0x0001);
        let pdu = req.encode().unwrap();
        assert_eq!(pdu, vec![0x04 as u8, 0x01, 0x02, 0x00, 0x01]);
    }

    #[test]
    fn decode_request() {
        let pdu: [u8; 5] = [0x04, 0xab, 0xcd, 0x00, 0x18];
        let req = Request::decode(&pdu).unwrap();
        assert_eq!(req.get_address(), 0xabcd);
        assert_eq!(req.get_quantity(), 0x0018);
    }

    #[test]
    fn encode_response() {
        let registers: [u16; 7] = [0x0123, 0x2345, 0xabcd, 0xedcb, 0x0000, 0xffff, 0x9876];
        let rsp = Response::new(&registers);
        let pdu = rsp.encode().unwrap();
        assert_eq!(pdu, vec![0x04 as u8, 0x0e, 0x01, 0x23, 0x23, 0x45, 0xab, 0xcd, 0xed, 0xcb, 0x00, 0x00, 0xff, 0xff, 0x98, 0x76]);
    }

    #[test]
    fn decode_response() {
        let pdu: [u8; 6] = [0x04, 0x04, 0xde, 0xad, 0xbe, 0xef];
        let rsp = Response::decode(&pdu).unwrap();
        assert_eq!(rsp.get_registers(), &vec![0xdead as u16, 0xbeef]);
    }
}