extern crate num;
#[macro_use]
extern crate num_derive;

mod error;
mod pdu;
mod transport;

pub use error::Error;
pub use pdu::RequestData;

pub use pdu::bit_access::read_coils::Request as ReadCoilsRequest;
pub use pdu::bit_access::read_dscr_in::Request as ReadDscrInRequest;
pub use pdu::hex_access::read_hld_reg::Request as ReadHldRegRequest;
pub use pdu::hex_access::read_in_reg::Request as ReadInRegRequest;
pub use pdu::bit_access::write_single_coil::Message as WriteSingleCoilRequest;
pub use pdu::hex_access::write_single_reg::Message as WriteSingleRegRequest;

pub use pdu::bit_access::read_coils::Response as ReadCoilsResponse;
pub use pdu::bit_access::read_dscr_in::Response as ReadDscrInResponse;
pub use pdu::hex_access::read_hld_reg::Response as ReadHldRegResponse;
pub use pdu::hex_access::read_in_reg::Response as ReadInRegResponse;
pub use pdu::bit_access::write_single_coil::Message as WriteSingleCoilResponse;
pub use pdu::hex_access::write_single_reg::Message as WriteSingleRegResponse;

pub use transport::Transport;
pub use transport::rtu::conn as rtu;
pub use transport::tcp::conn as tcp;
