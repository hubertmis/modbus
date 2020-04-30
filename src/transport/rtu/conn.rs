//! Modbus RTU over serial interface
 
use crate::error::Error;
use serialport::{SerialPort, SerialPortSettings, open_with_settings};
use std::convert::TryInto;
use std::ffi::OsStr;
use super::frame::Frame;
use super::super::Transport;

const BROADCAST_DST: u8 = 0;
 
#[derive(PartialEq)]
enum Role {
    Master,
    Slave(u8),
}

/// RTU transport for Modbus commands
/// 
/// This structure implements [Transport trait](Transport) that provides
/// functions needed to read and write Modbus functions using this transport.
pub struct Rtu {
    serial: Box<dyn SerialPort>,
    role: Role,
}

impl Rtu {
    /// Create a new RTU connection
    /// 
    /// This function opens serial port with [this](serialport::open_with_settings) function.
    /// 
    /// # Examples
    /// ```
    /// use serialport::{SerialPortSettings, DataBits, FlowControl, Parity, StopBits};
    /// use std::time::Duration;
    /// 
    /// let s = SerialPortSettings {
    ///     baud_rate: 115200,
    ///     data_bits: DataBits::Eight,
    ///     flow_control: FlowControl::None,
    ///     parity: Parity::None,
    ///     stop_bits: StopBits::Two,
    ///     timeout: Duration::from_millis(1),
    /// };
    /// let modbus = modbus::rtu::Rtu::conn("/dev/ttyUSB0", &s);
    /// ```
    pub fn conn<T: AsRef<OsStr> + ?Sized>(port: &T, settings: &SerialPortSettings) -> Result<Self, Error> {
        Ok(Rtu{serial: open_with_settings(port, settings)?, role: Role::Master})
    }

    fn write_pdu(&mut self, unit_id: u8, pdu: &[u8]) -> Result<(), Error> {
        let frame = Frame::new(unit_id, pdu);
        self.serial.write_all(&frame.encode()?)?;
        Ok(())
    }

    fn read_pdu(&mut self, expected_unit_id: u8) -> Result<Vec<u8>, Error> {
        let mut rsp_frame = Vec::new();
        let mut rsp_byte: [u8; 1] = [0];

        loop {
            match self.serial.read(&mut rsp_byte) {
                Ok(num_bytes) => {
                    assert_eq!(num_bytes, 1);
                    rsp_frame.push(rsp_byte[0]);
                }
                Err(err) => {
                    match err.kind() {
                        // TODO: Make timeout optional
                        std::io::ErrorKind::TimedOut => {
                            let frame = Frame::decode(&rsp_frame)?;
                            
                            if frame.is_address(expected_unit_id) {
                                return Ok(frame.get_pdu());
                            } else {
                                return Err(Error::InvalidData);
                            }
                        }
                        _ => { 
                            return Err(err.try_into().unwrap()); 
                        }
                    }
                }
            }
        }
    }

}

impl Transport for Rtu {
    type Dst = u8;
    type Stream = ();

    fn start_master(&mut self) -> Result<(), Error> {
        self.role = Role::Master;
        Ok(())
    }

    fn start_slave(&mut self, unit_id: u8) -> Result<(), Error> {
        match unit_id {
            1..=247 => {
                self.role = Role::Slave(unit_id);
                Ok(())
            }
            _ => Err(Error::InvalidValue)
        }
    }

    fn is_broadcast(dst: &Self::Dst) -> bool {
        *dst == BROADCAST_DST
    }

    fn write_req_pdu(&mut self, dst: &Self::Dst, pdu: &[u8]) -> Result<Self::Stream, Error> {
        self.write_pdu(*dst, pdu)?;
        Ok(())
    }

    fn read_rsp_pdu(&mut self, _: &mut Self::Stream, src: &Self::Dst) -> Result<Vec<u8>, Error> {
        self.read_pdu(*src)
    }

    fn read_req_pdu(&mut self) -> Result<(Vec<u8>, Self::Stream), Error> {
        if let Role::Slave(unit_id) = self.role {
            Ok((self.read_pdu(unit_id)?, ()))
        } else {
            Err(Error::InvalidValue)
        }
    }

    fn write_rsp_pdu(&mut self, _: &mut Self::Stream, pdu: &[u8]) -> Result<(), Error> {
        if let Role::Slave(unit_id) = self.role {
            self.write_pdu(unit_id, pdu)
        } else {
            Err(Error::InvalidValue)
        }
    }
}