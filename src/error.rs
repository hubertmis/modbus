use crate::pdu::ExceptionCode;
use serialport::Error as SerialError;
use std::convert::From;
use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;

/// The error types used by the modbus library
#[derive(Debug)]
pub enum Error {
    InvalidValue,

    TooShortData,
    InvalidData,
    InvalidDataLength,
    InvalidFunction,

    InvalidResponse,
    NoResponse,
    ExceptionResponse(ExceptionCode),

    InvalidRequest,
    MissingReqHandler,

    IoError(IoError),
    SerialError(SerialError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidValue => f.write_str("Invalid value"),
            Error::TooShortData => f.write_str("Too short data in the buffer"),
            Error::InvalidData => f.write_str("Invalid data"),
            Error::InvalidDataLength => f.write_str("Invalid data length"),
            Error::InvalidFunction => f.write_str("Invalid function code"),
            Error::InvalidResponse => f.write_str("Invalid response"),
            Error::NoResponse => f.write_str("No response"),
            Error::InvalidRequest => f.write_str("Invalid request"),
            Error::MissingReqHandler => f.write_str("Missing request handler for given request"),
            Error::ExceptionResponse(code) => f.write_str(&format!("Exception response: {}", code)),
            Error::IoError(error) => f.write_str(&format!("IO error: {}", error)),
            Error::SerialError(error) => f.write_str(&format!("Serial error: {}", error)),
        }
    }
}

impl StdError for Error {}

impl From<SerialError> for Error {
    fn from(error: SerialError) -> Self {
        Self::SerialError(error)
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Self::IoError(error)
    }
}
