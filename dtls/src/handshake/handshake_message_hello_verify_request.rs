#[cfg(test)]
mod handshake_message_hello_verify_request_test;

use super::*;
use crate::record_layer::record_layer_header::*;

use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

use util::Error;

/*
   The definition of HelloVerifyRequest is as follows:

   struct {
     ProtocolVersion server_version;
     opaque cookie<0..2^8-1>;
   } HelloVerifyRequest;

   The HelloVerifyRequest message type is hello_verify_request(3).

   When the client sends its ClientHello message to the server, the server
   MAY respond with a HelloVerifyRequest message.  This message contains
   a stateless cookie generated using the technique of [PHOTURIS].  The
   client MUST retransmit the ClientHello with the cookie added.

   https://tools.ietf.org/html/rfc6347#section-4.2.1
*/
#[derive(Clone, Debug, PartialEq)]
pub struct HandshakeMessageHelloVerifyRequest {
    version: ProtocolVersion,
    cookie: Vec<u8>,
}

impl HandshakeMessageHelloVerifyRequest {
    fn handshake_type() -> HandshakeType {
        HandshakeType::HelloVerifyRequest
    }

    pub fn marshal<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        if self.cookie.len() > 255 {
            return Err(ERR_COOKIE_TOO_LONG.clone());
        }

        writer.write_u8(self.version.major)?;
        writer.write_u8(self.version.minor)?;
        writer.write_u8(self.cookie.len() as u8)?;
        writer.write_all(&self.cookie)?;

        Ok(())
    }

    pub fn unmarshal<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let major = reader.read_u8()?;
        let minor = reader.read_u8()?;
        let cookie_length = reader.read_u8()?;
        let mut cookie = vec![];
        reader.read_to_end(&mut cookie)?;

        if cookie.len() < cookie_length as usize {
            return Err(ERR_BUFFER_TOO_SMALL.clone());
        }

        Ok(HandshakeMessageHelloVerifyRequest {
            version: ProtocolVersion { major, minor },
            cookie,
        })
    }
}
