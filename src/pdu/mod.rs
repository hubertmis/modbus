pub mod bit_access;
pub mod hex_access;

use crate::Error;
use num_enum::IntoPrimitive;
use std::convert::TryFrom;
use std::fmt;

const MAX_SIZE: usize = 253;

pub trait Function {
    fn encode(&self) -> Result<Vec<u8>, Error>;
    fn decode(data: &[u8]) -> Result<Self, Error> where Self: Sized;
}

pub trait Request: Function {
    type Rsp: Response;
}

pub trait Response: Function + Sized {
    fn get_exc_function_code() -> u8;

    fn decode_response(data: &[u8]) -> Result<Self, Error> {
        if let Ok(exc_code) = Self::decode_exc_rsp(data, Some(Self::get_exc_function_code())) {
            return Err(Error::ExceptionResponse(exc_code));
        }

        Self::decode(data)
    }

    fn decode_exc_rsp(data: &[u8], exp_fnc_code: Option<u8>) -> Result<ExceptionCode, Error> {
        if data.len() != 2 {
            return Err(Error::InvalidDataLength);
        }

        if let Some(exp_fnc_code) = exp_fnc_code {
            if data[0] != exp_fnc_code {
                return Err(Error::InvalidData);
            }
        }

        ExceptionCode::try_from(data[1])
    }
}

/// Setter is a trait for Modbus requests that expect the copy of the request as the response.
pub trait Setter: Request + Response + PartialEq {

}

#[derive(Clone, Copy, FromPrimitive, IntoPrimitive, PartialEq)]
#[repr(u8)]
pub enum FunctionCode {
    ReadCoils = 0x01,
    ReadDscrIn = 0x02,
    ReadHldReg = 0x03,
    ReadInReg = 0x04,
    WriteSingleCoil = 0x05,
    WriteSingleReg = 0x06,

    ExcReadCoils = 0x81,
    ExcReadDscrIn = 0x82,
    ExcReadHldReg = 0x83,
    ExcReadInReg = 0x84,
    ExcWriteSingleCoil = 0x85,
    ExcWriteSingleReg = 0x86,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExceptionCode {
    IllegalFunction                    = 0x01,
    IllegalDataAddress                 = 0x02,
    IllegalDataValue                   = 0x03,
    ServerDeviceFailure                = 0x04,
    Acknowledge                        = 0x05,
    ServerDeviceBusy                   = 0x06,
    MemoryParityError                  = 0x08,
    GatewayPathUnavailable             = 0x0A,
    GatewayTargetDeviceFailedToRespond = 0x0B,
}

impl fmt::Display for ExceptionCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExceptionCode::IllegalFunction => write!(f, "[exc] Illegal function"),
            ExceptionCode::IllegalDataAddress => write!(f, "[exc] Illegal data address"),
            ExceptionCode::IllegalDataValue => write!(f, "[exc] Illegal data value"),
            ExceptionCode::ServerDeviceFailure => write!(f, "[exc] Server device failure"),
            ExceptionCode::Acknowledge => write!(f, "[exc] Acknowledge"),
            ExceptionCode::ServerDeviceBusy => write!(f, "[exc] Server device busy"),
            ExceptionCode::MemoryParityError => write!(f, "[exc] Memory parity error"),
            ExceptionCode::GatewayPathUnavailable => write!(f, "[exc] Gateway path unavailable"),
            ExceptionCode::GatewayTargetDeviceFailedToRespond => write!(f, "[exc] Gateway target device failed to respond"),
        }
    }
}

impl TryFrom<u8> for ExceptionCode {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self, Error> {
        match v {
            x if x == ExceptionCode::IllegalFunction as u8 => Ok(ExceptionCode::IllegalFunction),
            x if x == ExceptionCode::IllegalDataAddress as u8 => Ok(ExceptionCode::IllegalDataAddress),
            x if x == ExceptionCode::IllegalDataValue as u8 => Ok(ExceptionCode::IllegalDataValue),
            x if x == ExceptionCode::ServerDeviceFailure as u8 => Ok(ExceptionCode::ServerDeviceFailure),
            x if x == ExceptionCode::Acknowledge as u8 => Ok(ExceptionCode::Acknowledge),
            x if x == ExceptionCode::ServerDeviceBusy as u8 => Ok(ExceptionCode::ServerDeviceBusy),
            x if x == ExceptionCode::MemoryParityError as u8 => Ok(ExceptionCode::MemoryParityError),
            x if x == ExceptionCode::GatewayPathUnavailable as u8 => Ok(ExceptionCode::GatewayPathUnavailable),
            x if x == ExceptionCode::GatewayTargetDeviceFailedToRespond as u8 => Ok(ExceptionCode::GatewayTargetDeviceFailedToRespond),
            _ => Err(Error::InvalidData),
        }
    }
}

/// Enumeration of Modbus request functions.
/// 
/// This enumeration is used to report received request in the Modbus slave mode.
#[derive(Debug)]
pub enum RequestData {
    ReadCoils(bit_access::read_coils::Request),
    ReadDscrIn(bit_access::read_dscr_in::Request),
    ReadHldReg(hex_access::read_hld_reg::Request),
    ReadInReg(hex_access::read_in_reg::Request),
    WriteSingleCoil(bit_access::write_single_coil::Message),
    WriteSingleReg(hex_access::write_single_reg::Message),
}

pub fn decode_req(pdu: &[u8]) -> Result<RequestData, Error> {
    if pdu.len() < 2 {
        return Err(Error::InvalidDataLength);
    }

    match num::FromPrimitive::from_u8(pdu[0]) {
        Some(FunctionCode::ReadCoils) => Ok(RequestData::ReadCoils(bit_access::read_coils::Request::decode(pdu)?)),
        Some(FunctionCode::ReadDscrIn) => Ok(RequestData::ReadDscrIn(bit_access::read_dscr_in::Request::decode(pdu)?)),
        Some(FunctionCode::ReadHldReg) => Ok(RequestData::ReadHldReg(hex_access::read_hld_reg::Request::decode(pdu)?)),
        Some(FunctionCode::ReadInReg) => Ok(RequestData::ReadInReg(hex_access::read_in_reg::Request::decode(pdu)?)),
        Some(FunctionCode::WriteSingleCoil) => Ok(RequestData::WriteSingleCoil(bit_access::write_single_coil::Message::decode(pdu)?)),
        Some(FunctionCode::WriteSingleReg) => Ok(RequestData::WriteSingleReg(hex_access::write_single_reg::Message::decode(pdu)?)),
        _ => Err(Error::InvalidData),
    }
}

/*
fn encode_exc_rsp(function_code: &FunctionCode, exception_code: &ExceptionCode) -> Result<Vec<u8>, Error> {
    let mut result = Vec::new();
    result.push(*function_code as u8);
    result.push(*exception_code as u8);

    Ok(result)
}
*/

#[cfg(test)]
mod tests {
}