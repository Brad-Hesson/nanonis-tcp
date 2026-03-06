use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    net::{TcpStream, ToSocketAddrs},
};

use crate::error::NanonisTcpResult;
use crate::fsm::NanonisTcpFsm;
use crate::{commands::Command, commands::*};

pub struct NanonisTcp {
    stream: TcpStream,
    buf: Vec<u8>,
}
impl NanonisTcp {
    pub async fn new(addr: impl ToSocketAddrs) -> std::io::Result<Self> {
        Ok(Self {
            stream: TcpStream::connect(addr).await?,
            buf: Vec::new(),
        })
    }
    pub async fn bias_set(&mut self, bias: f32) -> NanonisTcpResult<()> {
        self.call::<bias::Set>(&bias::SetArgs { bias }).await
    }
    pub async fn scan_props_get(&mut self) -> NanonisTcpResult<scan::PropsGetResponse> {
        self.call::<scan::PropsGet>(&()).await
    }
    pub async fn scan_buffer_get(&mut self) -> NanonisTcpResult<scan::BufferGetResponse> {
        self.call::<scan::BufferGet>(&()).await
    }
    pub async fn signals_names_get(&mut self) -> NanonisTcpResult<signals::NamesGetResponse> {
        self.call::<signals::NamesGet>(&()).await
    }
    pub async fn scan_frame_data_grab(
        &mut self,
        channel_index: usize,
        data_dir: usize,
    ) -> NanonisTcpResult<scan::FrameDataGrabResponse> {
        self.call::<scan::FrameDataGrab>(&scan::FrameDataGrabArgs {
            channel_index: channel_index as u32,
            data_dir: data_dir as u32,
        })
        .await
    }
    async fn call<C: Command>(&mut self, args: &C::Args) -> NanonisTcpResult<C::Response> {
        let fsm = NanonisTcpFsm::<C>::new(&mut self.buf, args)?;
        self.stream.write_all(fsm.bytes()).await?;
        self.stream.flush().await?;
        let mut fsm = fsm.prepare_for_header()?;
        self.stream.read_exact(fsm.bytes_mut()).await?;
        let mut fsm = fsm.prepare_for_body()?;
        self.stream.read_exact(fsm.bytes_mut()).await?;
        fsm.parse_response()
    }
}
