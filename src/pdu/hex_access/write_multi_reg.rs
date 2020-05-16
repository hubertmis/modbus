use crate::Error;
use crate::pdu::{Function, FunctionCode, Request as ReqT, Response as RspT, Setter};
use std::convert::TryInto;

const MIN_QUANTITY: usize = 1;
const MAX_QUANTITY: usize = 123;

/// Write Multiple Registers request function
#[derive(Debug, PartialEq)]
pub struct Request {
    address: u16,
    values: Vec<u16>,
}

impl Request {
    /// Create a new Write Multiple Registers request function
    /// 
    /// # Examples
    /// ```
    /// let req = modbus::WriteMultiRegRequest::new(0xabcd, &vec![0xcafe, 0xface]);
    /// ```
    pub fn new(address: u16, values: &[u16]) -> Self {
        assert!(values.len() >= MIN_QUANTITY);
        assert!(values.len() <= MAX_QUANTITY);

        Request{address, values: Vec::from(values)}
    }

    /// Get address of the starting register from the Write Multiple Reigsters request function
    /// 
    /// # Examples
    /// ```
    /// let address = 0x0abc;
    /// let req = modbus::WriteMultiRegRequest::new(address, &vec![0x0000, 0x0001]);
    /// assert_eq!(req.get_address(), address);
    /// ```
    pub fn get_address(&self) -> u16 {
        self.address
    }

    /// Get values from the Write Multiple Registers request function
    /// 
    /// # Examples
    /// ```
    /// let values = vec![0x0123, 0x1234, 0x2345];
    /// let req = modbus::WriteMultiRegRequest::new(0xfedc, &values);
    /// assert_eq!(&Vec::from(req.get_values()), &values);
    /// ```
    pub fn get_values(&self) -> &[u16] {
        &self.values
    }
}

impl Function for Request {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        match self.values.len() {
            MIN_QUANTITY..=MAX_QUANTITY => {
                let mut result = Vec::new();
                result.push(FunctionCode::WriteMultiReg as u8);
                result.append(&mut self.address.to_be_bytes().to_vec());
                result.append(&mut (self.values.len() as u16).to_be_bytes().to_vec());
                result.push((self.values.len() as u8) * 2);

                for val in &self.values {
                    result.append(&mut val.to_be_bytes().to_vec());
                }

                Ok(result)
            }
            _ => Err(Error::InvalidValue)
        }
    }

    fn decode(data: &[u8]) -> Result<Self, Error> {
        if data.len() < 6 {
            return Err(Error::InvalidDataLength);
        }
        if data[0] != FunctionCode::WriteMultiReg as u8 {
            return Err(Error::InvalidData);
        }
        
        let address = u16::from_be_bytes(data[1..=2].try_into().unwrap());
        let quantity = u16::from_be_bytes(data[3..=4].try_into().unwrap());
        let data_cnt = data[5];

        if data_cnt as u16 != quantity * 2 {
            return Err(Error::InvalidDataLength);
        }
        if (quantity as usize) < MIN_QUANTITY || (quantity as usize) > MAX_QUANTITY {
            return Err(Error::InvalidData);
        }

        let mut values = Vec::with_capacity(quantity as usize);

        for i in 0..quantity {
            let val_idx = (6 + i * 2) as usize;
            values.push(u16::from_be_bytes(data[val_idx..=val_idx+1].try_into().unwrap()))
        }

        Ok(Self{address, values})
    }
}

impl ReqT for Request {
    type Rsp = Response;
}

impl Setter for Request {
    fn create_expected_response(&self) -> Self::Rsp {
        Response::new(self.address, self.values.len() as u16)
    }
}

/// Write Multiple Registers response function
#[derive(Debug, PartialEq)]
pub struct Response {
    address: u16,
    quantity: u16,
}

impl Response {
    /// Create a new Write Multiple Registers response function
    /// 
    /// # Examples
    /// ```
    /// let rsp = modbus::WriteMultiRegResponse::new(0xabcd, 0x007B);
    /// ```
    pub fn new(address: u16, quantity: u16) -> Self {
        assert!(quantity as usize >= MIN_QUANTITY);
        assert!(quantity as usize <= MAX_QUANTITY);

        Self{address, quantity}
    }

    /// Get address of the starting register from the Write Multiple Reigsters response function
    /// 
    /// # Examples
    /// ```
    /// let address = 0x0abc;
    /// let rsp = modbus::WriteMultiRegResponse::new(address, 0x0001);
    /// assert_eq!(rsp.get_address(), address);
    /// ```
    pub fn get_address(&self) -> u16 {
        self.address
    }

    /// Get quantity from the Write Multiple Registers response function
    /// 
    /// # Examples
    /// ```
    /// let quantity = 0x0070;
    /// let rsp = modbus::WriteMultiRegResponse::new(0xfedc, quantity);
    /// assert_eq!(rsp.get_quantity(), quantity);
    /// ```
    pub fn get_quantity(&self) -> u16 {
        self.quantity
    }
}

impl Function for Response {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        match self.quantity as usize {
            MIN_QUANTITY..=MAX_QUANTITY => {
                let mut result = Vec::new();
                result.push(FunctionCode::WriteMultiReg as u8);
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
        if data[0] != FunctionCode::WriteMultiReg as u8 {
            return Err(Error::InvalidData);
        }
        
        let address = u16::from_be_bytes(data[1..=2].try_into().unwrap());
        let quantity = u16::from_be_bytes(data[3..=4].try_into().unwrap());

        Ok(Self{address, quantity})
    }
}

impl RspT for Response {
    fn get_exc_function_code() -> u8 {
        FunctionCode::ExcWriteMultiReg.try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_request() {
        let req = Request::new(0xdead, &vec![0xfade, 0xface, 0x0000, 0x0001]);
        let pdu = req.encode().unwrap();
        let expected_pdu = vec![0x10, 0xde, 0xad, 0x00, 0x04, 0x08, 
                                0xfa, 0xde, 0xfa, 0xce, 0x00, 0x00, 0x00, 0x01];

        assert_eq!(pdu, expected_pdu);
    }

    #[test]
    fn test_encode_response() {
        let rsp = Response::new(0xffff, 0x0072);
        let pdu = rsp.encode().unwrap();
        let expected_pdu = vec![0x10, 0xff, 0xff, 0x00, 0x72];

        assert_eq!(pdu, expected_pdu);
    }

    #[test]
    fn test_decode_request() {
        let pdu = vec![0x10, 0x00, 0x00, 0x00, 0x02, 0x04, 0x01, 0x02, 0xfe, 0xfd];
        let req = Request::decode(&pdu).unwrap();
        let expected_req = Request::new(0x0000, &vec![0x0102, 0xfefd]);

        assert_eq!(req, expected_req);
    }

    #[test]
    fn test_decode_invalid_request() {
        let pdu = vec![0x11, 0x01, 0x23, 0x00, 0x01, 0x02, 0x11, 0x12];
        let err = Request::decode(&pdu).err().unwrap();
        match err {
            Error::InvalidData => {}
            _ => panic!(format!("Expected InvalidData, but got {:?}", err)),
        }
    }

    #[test]
    fn test_decode_response() {
        let pdu = vec![0x10, 0x01, 0x23, 0x00, 0x65];
        let rsp = Response::decode(&pdu).unwrap();
        let expected_rsp = Response::new(0x0123, 0x0065);

        assert_eq!(rsp, expected_rsp);
    }
}