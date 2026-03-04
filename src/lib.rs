use std::{
    fmt::Debug,
    io::{Cursor, Read, Seek, Write},
    net::{TcpStream, ToSocketAddrs},
    thread::panicking,
};

use binrw::{BinRead, BinWrite, binread, binwrite, io::NoSeek};

use crate::commands::{BiasSetReq, ScanFrameDataGrab};

pub mod commands;

pub trait Command {
    const NAME: &'static str;

    type Request: for<'a> BinWrite<Args<'a> = ()>;
    type Response: for<'a> BinRead<Args<'a> = ()>;

    fn write_request<W: Write + Seek>(
        request: &Self::Request,
        writer: &mut W,
    ) -> binrw::BinResult<()> {
        let mut packet = RequestPacket {
            name: Self::NAME,
            response_back: true,
            body: Vec::new(),
        };
        request.write_be(&mut Cursor::new(&mut packet.body))?;
        packet.write_be(writer)?;
        Ok(())
    }
    fn read_response_header<R: Read + Seek>(reader: &mut R) -> binrw::BinResult<ResponseHeader> {
        let header = ResponseHeader::read_be(reader)?;
        Ok(header)
    }
    fn read_response_body<R: Read + Seek>(
        reader: &mut R,
    ) -> binrw::BinResult<Result<Self::Response, NanonisError>> {
        let packet = ResponseBody::read_be(reader)?;
        Ok(match packet.error_status {
            0 => Ok(packet.body),
            _ => Err(NanonisError {
                desc: packet.error_description,
            }),
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Nanonis API error: {desc}")]
pub struct NanonisError {
    desc: String,
}

#[binwrite]
#[derive(Debug)]
struct RequestPacket {
    #[bw(map = |s: &&str| s.as_bytes())]
    #[bw(pad_size_to = 32)]
    name: &'static str,
    #[bw(calc = body.len() as i32)]
    body_len: i32,
    #[bw(map = |b: &bool| u16::from(*b))]
    response_back: bool,
    #[bw(pad_before = 2)]
    body: Vec<u8>,
}

#[binread]
#[derive(Debug)]
pub struct ResponseHeader {
    #[br(map = |b: [u8; 32]| String::from_utf8_lossy(&b).to_string())]
    name: String,
    #[br(pad_after = 4)]
    body_size: i32,
}

#[binread]
#[derive(Debug)]
struct ResponseBody<B: for<'a> BinRead<Args<'a> = ()>> {
    body: B,
    error_status: u32,
    #[br(temp)]
    error_len: i32,
    #[br(count = error_len)]
    #[br(map = |b: Vec<u8>| String::from_utf8_lossy(&b).to_string())]
    error_description: String,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn feature() {
        let mut nanonis = Nanonis::new("127.0.0.1:6502");
        let idxs = dbg!(nanonis.scan_buffer_get()).unwrap().channel_indexes;
        let names = dbg!(nanonis.signals_names_get()).unwrap().names;
        dbg!(nanonis.scan_props_get()).unwrap();
        for i in idxs {
            println!("{}", names[i as usize]);
        }
    }
}

pub struct Nanonis {
    stream: TcpStream,
}
impl Nanonis {
    pub fn new(addr: impl ToSocketAddrs) -> Self {
        Self {
            stream: TcpStream::connect(addr).unwrap(),
        }
    }
    pub fn bias_set(&mut self, bias: f32) -> Result<(), NanonisError> {
        self.call::<commands::BiasSet>(&BiasSetReq { bias })
    }
    pub fn scan_props_get(&mut self) -> Result<commands::ScanPropsGetResp, NanonisError> {
        self.call::<commands::ScanPropsGet>(&())
    }
    pub fn scan_buffer_get(&mut self) -> Result<commands::ScanBufferGetResp, NanonisError> {
        self.call::<commands::ScanBufferGet>(&())
    }
    pub fn signals_names_get(&mut self) -> Result<commands::SignalsNamesGetResp, NanonisError> {
        self.call::<commands::SignalsNamesGet>(&())
    }

    fn call<C: Command>(&mut self, args: &C::Request) -> Result<C::Response, NanonisError> {
        let mut write_buf = Vec::new();
        C::write_request(args, &mut Cursor::new(&mut write_buf)).unwrap();
        self.stream.write_all(&write_buf).unwrap();
        self.stream.flush().unwrap();
        let mut header_buf = vec![0; 40];
        self.stream.read_exact(&mut header_buf).unwrap();
        let header = C::read_response_header(&mut Cursor::new(&mut header_buf)).unwrap();
        let mut body_buf = vec![0; header.body_size as usize];
        self.stream.read_exact(&mut body_buf).unwrap();
        let data = C::read_response_body(&mut Cursor::new(&mut body_buf)).unwrap();
        data
    }
}
