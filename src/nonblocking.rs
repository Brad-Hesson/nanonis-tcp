use std::time::Duration;

use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    net::{TcpStream, ToSocketAddrs},
};

use crate::{ActionType, LineDir, error::NanonisTcpResult};
use crate::{ScanDir, fsm::NanonisTcpFsm};
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
    pub async fn scan_action(&mut self, action: ActionType, dir: ScanDir) -> NanonisTcpResult<()> {
        self.call::<scan::Action>(&scan::ActionArgs { action, dir })
            .await
    }
    pub async fn scan_status_get(&mut self) -> NanonisTcpResult<scan::StatusGetResponse> {
        self.call::<scan::StatusGet>(&()).await
    }
    pub async fn scan_wait_end_of_line(
        &mut self,
        timeout: Option<Duration>,
    ) -> NanonisTcpResult<scan::WaitEndOfLineResponse> {
        self.call::<scan::WaitEndOfLine>(&scan::WaitEndOfLineArgs { timeout })
            .await
    }
    pub async fn scan_frame_get(&mut self) -> NanonisTcpResult<scan::FrameGetResponse> {
        self.call::<scan::FrameGet>(&()).await
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
        channel_index: u32,
        data_dir: LineDir,
    ) -> NanonisTcpResult<scan::FrameDataGrabResponse> {
        self.call::<scan::FrameDataGrab>(&scan::FrameDataGrabArgs {
            channel_index,
            data_dir,
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
