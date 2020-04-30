//! Modbus over TCP/IP
 
use crate::error::Error;
use std::convert::TryInto;
use std::io::prelude::*;
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::time::Duration;
use super::frame::Frame;
use super::super::Transport;

const TCP_PORT: u16 = 5020;
const BROADCAST_UNIT_ID: u8 = 0;

/// Structure describing destination node for TCP/IP Modbus functions
pub struct Dst {
    ip_addr: IpAddr,
    unit_id: u8,
}

impl Dst {
    /// Create a new TCP/IP destination description
    /// 
    /// # Examples
    /// ```
    /// # use std::net::{IpAddr, Ipv4Addr};
    /// let dst = modbus::tcp::Dst::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10);
    /// ```
    pub fn new(ip_addr: IpAddr, unit_id: u8) -> Self {
        Self {ip_addr, unit_id}
    }
}

/// TCP/IP transport for the Modbus commands
/// 
/// This structure implements [Transport trait](Transport) that provides
/// functions needed to read and write Modbus functions using this transport.
pub struct Tcp {
    listener: Option<TcpListener>,
    unit_id: u8,
}

impl Tcp {
    /// Create a new instance of the Modbus transport
    /// 
    /// # Examples
    /// ```
    /// let modbus = modbus::tcp::Tcp::new();
    /// ```
    pub fn new() -> Self {
        Self {listener: None, unit_id: 255}
    }

    fn connect(addr: &SocketAddr) -> Result<TcpStream, Error> {
        let stream = TcpStream::connect_timeout(addr, Duration::from_secs(1))?;
        stream.set_read_timeout(Some(Duration::from_secs(1)))?;
        Ok(stream)
    }

    fn read_pdu(stream: &mut TcpStream, expected_unit_id: u8) -> Result<Vec<u8>, Error> {
        let mut frame_pdu = Vec::new();
        let mut byte: [u8; 1] = [0];

        loop {
            match stream.read(&mut byte) {
                Ok(0) => return Err(Error::InvalidDataLength),
                Ok(1) => frame_pdu.push(byte[0]),
                Ok(_) => panic!("Invalid number of bytes received"),
                Err(err) => {
                    return Err(err.try_into().unwrap()); 
                }
            }

            match Frame::decode(&frame_pdu) {
                Err(Error::TooShortData) => {},
                Ok(frame) => {
                    if frame.get_unit_id() == expected_unit_id {
                        return Ok(Vec::from(frame.get_pdu()));
                    } else {
                        return Err(Error::InvalidData);
                    }
                }
                Err(err) => panic!("Unexpected parsing error: {:?}", err),
            }
        }
    }

    fn write_pdu(stream: &mut TcpStream, pdu: &[u8], unit_id: u8) -> Result<(), Error> {
        let frame = Frame::new(unit_id, pdu);
        stream.write_all(&frame.encode()?)?;
        Ok(())
    }
}

impl Transport for Tcp {
    type Dst = Dst;
    type Stream = TcpStream;

    fn start_master(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn start_slave(&mut self, unit_id: u8) -> Result<(), Error> {
        self.unit_id = unit_id;
        self.listener = Some(TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], TCP_PORT)))?);
        Ok(())
    }

    fn is_broadcast(dst: &Self::Dst) -> bool {
        dst.unit_id == BROADCAST_UNIT_ID
    }

    fn write_req_pdu(&mut self, dst: &Self::Dst, pdu: &[u8]) -> Result<Self::Stream, Error> {
        let peer_addr = SocketAddr::from((dst.ip_addr, TCP_PORT));
        let mut stream = Self::connect(&peer_addr)?;

        Self::write_pdu(&mut stream, pdu, dst.unit_id)?;
        Ok(stream)
    }

    fn read_rsp_pdu(&mut self, stream: &mut Self::Stream, src: &self::Dst) -> Result<Vec<u8>, Error>
    {
        // TODO: Timeout
        Self::read_pdu(stream, src.unit_id)
    }

    fn read_req_pdu(&mut self) -> Result<(Vec<u8>, Self::Stream), Error> {
        if let Some(listener) = &self.listener {
            let (mut socket, _addr) = listener.accept()?;

            Ok((Self::read_pdu(&mut socket, self.unit_id)?, socket))
        }
        else {
            Err(Error::InvalidValue)
        }
    }

    fn write_rsp_pdu(&mut self, stream: &mut Self::Stream, pdu: &[u8]) -> Result<(), Error> {
        Self::write_pdu(stream, pdu, self.unit_id)
    }
}

#[cfg(test)]
mod tests {
    //use crate::ReadCoilsResponse;
    //use super::*;
    //use std::net::{IpAddr, Ipv4Addr};

    /*
    #[test]
    fn test_tcp_listener() {
        let mut tcp = Tcp::new();
        let rsp = tcp.start_slave(10);
        println!("{:?}", rsp);
        let req = tcp.read_unsolicited_pdu()
        println!("{:?}", req);
        assert_eq!(true, false);
    }
    */
}
