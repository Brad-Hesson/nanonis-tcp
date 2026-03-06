use std::{
    borrow::Cow,
    fmt::Debug,
    io::{Cursor, Read, Write},
    net::{TcpStream, ToSocketAddrs},
};

use macro_rules_attribute::apply;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    codec::{CodecRead, CodecReadDerive, CodecWrite},
    error::NanonisResult,
};

mod codec;
pub mod commands;
pub mod error;

pub trait Command {
    const NAME: &'static str;

    type Args: CodecWrite;
    type Response: CodecRead;

    fn write(args: &Self::Args, writer: &mut impl Write) -> NanonisResult<usize> {
        let header = Header {
            name: Self::NAME.into(),
            body_size: args.codec_len(),
            response: true,
        };
        header.codec_write(writer)?;
        args.codec_write(writer)?;
        Ok(40 + header.body_size)
    }

    fn read(reader: &mut impl Read) -> NanonisResult<Self::Response> {
        let header = Header::codec_read(reader)?;
        assert_eq!(header.name, Self::NAME, "response name did not match");
        let mut buf = vec![0u8; header.body_size];
        reader.read_exact(&mut buf)?;
        let mut cur = Cursor::new(&mut buf);
        let response = Self::Response::codec_read(&mut cur)?;
        let error = ErrorFooter::codec_read(&mut cur)?;
        if error.status != 0 {
            return Err(error::NanonisError::Api(error.description));
        }
        assert_eq!(
            cur.position() as usize,
            header.body_size,
            "response body length did not match"
        );
        Ok(response)
    }
}

#[derive(Debug)]
struct Header {
    name: Cow<'static, str>,
    body_size: usize,
    response: bool,
}
impl CodecRead for Header {
    fn codec_read(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        let mut name_buf = [0u8; 32];
        reader.read_exact(&mut name_buf)?;
        let name_len = name_buf.iter().position(|b| *b == 0).unwrap_or(32);
        let name = str::from_utf8(&name_buf[..name_len])
            .map_err(std::io::Error::other)?
            .to_string();
        let body_size = i32::codec_read(reader)?;
        let response = u16::codec_read(reader)?;
        let _ = u16::codec_read(reader)?;
        Ok(Self {
            name: name.into(),
            body_size: body_size as usize,
            response: response != 0,
        })
    }
}
impl CodecWrite for Header {
    fn codec_write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        let mut name_buf = [0u8; 32];
        name_buf[0..self.name.len()].copy_from_slice(self.name.as_bytes());
        writer.write_all(&name_buf)?;
        i32::codec_write(&(self.body_size as i32), writer)?;
        u16::codec_write(&(self.response as u16), writer)?;
        u16::codec_write(&0, writer)?;
        Ok(())
    }

    fn codec_len(&self) -> usize {
        40
    }
}

#[derive(Debug)]
#[apply(CodecReadDerive)]
struct ErrorFooter {
    status: u32,
    description: String,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn blocking() {
        let mut nanonis = Nanonis::new("127.0.0.1:6502");
        let props = nanonis.call::<commands::ScanPropsGet>(&()).unwrap();
        nanonis.scan_frame_data_grab(0, 0).ok();
        nanonis.scan_frame_data_grab(1, 0).ok();
        println!("{props:?}")
    }

    #[tokio::test]
    async fn asink() {
        let mut nanonis = Nanonis::new("127.0.0.1:6502");
        loop{
            let props = nanonis.call::<commands::ScanPropsGet>(&());
            nanonis.scan_frame_data_grab(0, 0).ok();
            nanonis.scan_frame_data_grab(1, 0).ok();
            println!("{props:?}")
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
    // pub fn bias_set(&mut self, bias: f32) -> Result<(), NanonisError> {
    //     self.call::<commands::BiasSet>(&BiasSetReq { bias })
    // }
    // pub fn scan_props_get(&mut self) -> Result<commands::ScanPropsGetResp, NanonisError> {
    //     self.call::<commands::ScanPropsGet>(&())
    // }
    pub fn scan_buffer_get(&mut self) -> NanonisResult<commands::ScanBufferGetResponse> {
        self.call::<commands::ScanBufferGet>(&())
    }
    pub fn signals_names_get(&mut self) -> NanonisResult<commands::SignalsNamesGetResponse> {
        self.call::<commands::SignalsNamesGet>(&())
    }
    pub fn scan_frame_data_grab(
        &mut self,
        channel_index: usize,
        data_dir: usize,
    ) -> NanonisResult<commands::ScanFrameDataGrabResponse> {
        self.call::<commands::ScanFrameDataGrab>(&commands::ScanFrameDataGrabArgs {
            channel_index: channel_index as u32,
            data_dir: data_dir as u32,
        })
    }
    fn call<C: Command>(&mut self, args: &C::Args) -> NanonisResult<C::Response> {
        C::write(args, &mut self.stream)?;
        C::read(&mut self.stream)
    }
}

pub struct NanonisAsync {
    stream: tokio::net::TcpStream,
    buf: Box<[u8]>,
}
impl NanonisAsync {
    pub async fn new(addr: impl tokio::net::ToSocketAddrs) -> Self {
        Self {
            stream: tokio::net::TcpStream::connect(addr).await.unwrap(),
            buf: vec![0u8; 1024 * 1024].into_boxed_slice(),
        }
    }
    pub async fn call<C: Command>(&mut self, args: &C::Args) -> NanonisResult<C::Response> {
        let written = C::write(args, &mut &mut self.buf[..])?;
        self.stream.write_all(&self.buf[..written]).await?;
        self.stream.flush().await?;
        let written = self.stream.read_exact(&mut self.buf[..40]).await?;
        let header = Header::codec_read(&mut &self.buf[..written])?;
        let written = self
            .stream
            .read_exact(&mut self.buf[..header.body_size])
            .await?;
        let mut body = &self.buf[..written];
        let response = C::Response::codec_read(&mut body)?;
        let error = ErrorFooter::codec_read(&mut body)?;
        if error.status != 0 {
            return Err(error::NanonisError::Api(error.description));
        }
        Ok(response)
    }
}
