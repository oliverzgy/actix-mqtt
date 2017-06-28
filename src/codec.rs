use super::{Packet, read_packet, WritePacketExt, calc_remaining_length};
use tokio_io::codec::{Decoder, Encoder};
use bytes::{BytesMut, BufMut};
use nom::IError;
use std::io::{Error, ErrorKind};

#[derive(Default)]
pub struct Codec;

impl Decoder for Codec {
    type Item = Packet;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let packet_len: usize;
        let payload_len: usize;
        let mut p: Packet;
        match read_packet(src) {
            Ok((rest, packet, size)) => {
                packet_len = (rest.as_ptr() as usize) - (src.as_ptr() as usize);
                payload_len = size;
                p = packet;
            }
            // todo: derive error
            Err(IError::Error(_)) => return Err(Error::new(ErrorKind::Other, "oops")),
            Err(IError::Incomplete(_)) => return Ok(None),
        };
        let mut parsed = src.split_to(packet_len);
        // pull payload for publish packet if it was deferred
        match p {
            Packet::Publish {ref mut payload, ..} if payload_len > 0 => {
                let len = parsed.len();
                *payload = parsed.split_off(len - payload_len).freeze();
            },
            _ => {}
        }
        Ok(Some(p))
    }
}

impl Encoder for Codec {
    type Item = Packet;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let content_size = calc_remaining_length(&item);
        dst.reserve(content_size + 5);
        dst.writer().write_packet(&item);
        Ok(())
    }
}
