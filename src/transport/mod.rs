pub mod rtu;
pub mod tcp;

use crate::error::Error;
use crate::pdu::{Request, Response, Setter, RequestData, decode_req};

/// The trait implemented by Modbus protocol link layers 
pub trait Transport {
    /// Type describing message destination
    type Dst;
    /// Stream used to read or write messages in during data exchange
    type Stream;

    /// Enable Modbus master mode for given transport.
    fn start_master(&mut self) -> Result<(), Error>;
    /// Enable Modbus slave mode for given transport.
    fn start_slave(&mut self, unit_id: u8) -> Result<(), Error>;

    /// Verify if given destination is broadcast.
    fn is_broadcast(dst: &Self::Dst) -> bool;

    /// Write PDU of a request frame through given transport.
    /// 
    /// This method shall be used only in master mode.
    /// This method returns Stream that shall be used to read response.
    fn write_req_pdu(&mut self, dst: &Self::Dst, pdu: &[u8]) -> Result<Self::Stream, Error>;

    /// Read PDU of a response frame through given transport.
    /// 
    /// This method shall be used only in master mode.
    fn read_rsp_pdu(&mut self, stream: &mut Self::Stream, src: &Self::Dst) -> Result<Vec<u8>, Error>;

    /// Read PDU of a request frame through given transport.
    /// 
    /// This method shall be used only is the slave mode.
    fn read_req_pdu(&mut self) -> Result<(Vec<u8>, Self::Stream), Error>;

    /// Write PDU of a response frame through given transport.
    /// 
    /// This method shall be used only in the slave mode.
    fn write_rsp_pdu(&mut self, stream: &mut Self::Stream, pdu: &[u8]) -> Result<(), Error>;

    /// Write a request frame and read a response frame.
    /// 
    /// # Examples
    /// ```no_run
    /// # use modbus::Transport;
    /// # use std::net::{IpAddr, Ipv4Addr};
    /// #
    /// let mut mb = modbus::tcp::Tcp::new();
    /// let dst = modbus::tcp::Dst::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10);
    /// let req = modbus::ReadCoilsRequest::new(0x0123, 0x0002);
    /// let rsp = mb.write_req_read_rsp(&dst, &req);
    /// ```
    fn write_req_read_rsp<Req: Request>(&mut self, dst: &Self::Dst, req: &Req) -> Result<Option<Req::Rsp>, Error> {
        let req_pdu: Vec<u8> = req.encode()?;
        let mut stream = self.write_req_pdu(dst, &req_pdu)?;

        if Self::is_broadcast(dst) {
            Ok(None)
        } else {
            let rsp_pdu = self.read_rsp_pdu(&mut stream, dst)?;
            Ok(Some(Req::Rsp::decode_response(&rsp_pdu)?))
        }
    }

    /// Write a setter request and read a response frame.
    /// 
    /// This function handles unexpected responses
    /// 
    /// # Examples
    /// ```no_run
    /// # use modbus::Transport;
    /// # use std::net::{IpAddr, Ipv4Addr};
    /// #
    /// let mut mb = modbus::tcp::Tcp::new();
    /// let dst = modbus::tcp::Dst::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10);
    /// let req = modbus::WriteSingleCoilRequest::new(0x0123, true);
    /// mb.write_req_read_rsp(&dst, &req).unwrap();
    /// ```
    fn write_setter_req<Req: Setter>(&mut self, dst: &Self::Dst, req: &Req) -> Result<(), Error> {
        let req_pdu: Vec<u8> = req.encode()?;
        let mut stream = self.write_req_pdu(dst, &req_pdu)?;

        if Self::is_broadcast(dst) {
            Ok(())
        } else {
            let rsp_pdu = self.read_rsp_pdu(&mut stream, dst)?;
            let rsp = Req::decode_response(&rsp_pdu)?;

            if req == &rsp {
                Ok(())
            } else {
                Err(Error::InvalidData)
            }
        }

    }

    /// Read a request frame.
    /// 
    /// This method with [Transport::write_rsp] are the main functionality in the Modbus slave mode.
    /// 
    /// # Examples
    /// ```no_run
    /// use modbus::Transport;
    /// 
    /// let mut mb = modbus::tcp::Tcp::new();
    /// mb.start_slave(10).unwrap();
    /// let (req, stream) = mb.read_req().unwrap();
    /// ```
    fn read_req(&mut self) -> Result<(RequestData, Self::Stream), Error> {
        let (req_pdu, stream) = self.read_req_pdu()?;
        let req_data = decode_req(&req_pdu)?;
        Ok((req_data, stream))
    }

    /// Write a response frame.
    /// 
    /// Call to this method shall follow [Transport::read_req] in the Modbus slave mode.
    /// 
    /// # Examples
    /// ```no_run
    /// use modbus::Transport;
    /// 
    /// let mut mb = modbus::tcp::Tcp::new();
    /// mb.start_slave(10).unwrap();
    /// let (req, stream) = mb.read_req().unwrap();
    /// 
    /// if let modbus::RequestData::ReadCoils(request) = req {
    ///     let result = mb.write_rsp(stream, modbus::ReadCoilsResponse::new(&[true, false]));
    /// }
    /// ```
    fn write_rsp<Rsp: Response>(&mut self, mut stream: Self::Stream, response: Rsp) -> Result<(), Error> {
        self.write_rsp_pdu(&mut stream, &response.encode()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ReadCoilsResponse;

    /*
    use crate::ReadCoilsRequest;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_tcp_master() {
        let mut mb = tcp::conn::Tcp::new();
        let dst = tcp::conn::Dst::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10);
        let req = ReadCoilsRequest::new(0x0123, 0x0002);

        let rsp = mb.write_req_read_rsp(&dst, &req);
        println!("{:?}", rsp);
        assert_eq!(false, true);
    }

    #[test]
    fn test_tcp_slave() {
        let mut mb = tcp::conn::Tcp::new();
        mb.start_slave(10).unwrap();
        let (req, stream) = mb.read_req().unwrap();
        let res = mb.write_rsp(stream, ReadCoilsResponse::new(&[true, false]));

        if let Err(error) = res {
            panic!("Error during writing response: {:?}", error);
        }
        println!("{:?}", req);
        assert_eq!(false, true);
    }
    */

    #[test]
    fn test_reading_coils() {
        let exc_fn_code = ReadCoilsResponse::get_exc_function_code();
        assert_eq!(0x81, exc_fn_code);

        let err = ReadCoilsResponse::decode_response(&[0x81, 0x01]);
        if let Err(error) = err {
            match error {
                Error::ExceptionResponse(_) => {}
                _ => panic!("Invalid error reported"),
            }
        }
        else {
            panic!("Expected error, but got Ok result");
        }
    }
}
