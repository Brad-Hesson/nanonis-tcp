use std::io::{Read as _, Write as _};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::codec::{ActionType, LineDir, ScanDir};
use crate::error::NanonisTcpResult;
use crate::fsm::NanonisTcpFsm;
use crate::{commands::Command, commands::*};

pub struct NanonisTcp {
    stream: TcpStream,
    buf: Vec<u8>,
}
impl NanonisTcp {
    pub fn new(addr: impl ToSocketAddrs) -> std::io::Result<Self> {
        Ok(Self {
            stream: TcpStream::connect(addr)?,
            buf: Vec::new(),
        })
    }
    pub fn piezo_range_get(&mut self) -> NanonisTcpResult<piezo::RangeGetResponse> {
        self.call::<piezo::RangeGet>(&())
    }
    pub fn scan_action(&mut self, action: ActionType, dir: ScanDir) -> NanonisTcpResult<()> {
        self.call::<scan::Action>(&scan::ActionArgs { action, dir })
    }
    pub fn scan_status_get(&mut self) -> NanonisTcpResult<scan::StatusGetResponse> {
        self.call::<scan::StatusGet>(&())
    }
    pub fn scan_wait_end_of_line(
        &mut self,
        timeout: Option<Duration>,
    ) -> NanonisTcpResult<scan::WaitEndOfLineResponse> {
        self.call::<scan::WaitEndOfLine>(&scan::WaitEndOfLineArgs { timeout })
    }
    pub fn scan_wait_end_of_scan(
        &mut self,
        timeout: Option<Duration>,
    ) -> NanonisTcpResult<scan::WaitEndOfScanResponse> {
        self.call::<scan::WaitEndOfScan>(&scan::WaitEndOfScanArgs { timeout })
    }
    pub fn scan_frame_set(
        &mut self,
        center_x: f32,
        center_y: f32,
        width: f32,
        height: f32,
        angle: f32,
    ) -> NanonisTcpResult<()> {
        self.call::<scan::FrameSet>(&scan::FrameSetArgs {
            center_x,
            center_y,
            width,
            height,
            angle,
        })
    }
    pub fn scan_frame_get(&mut self) -> NanonisTcpResult<scan::FrameGetResponse> {
        self.call::<scan::FrameGet>(&())
    }
    pub fn scan_xy_pos_get(
        &mut self,
        wait_newest: bool,
    ) -> NanonisTcpResult<scan::XYPosGetResponse> {
        self.call::<scan::XYPosGet>(&scan::XYPosGetArgs { wait_newest })
    }
    pub fn bias_set(&mut self, bias: f32) -> NanonisTcpResult<()> {
        self.call::<bias::Set>(&bias::SetArgs { bias })
    }
    pub fn scan_props_get(&mut self) -> NanonisTcpResult<scan::PropsGetResponse> {
        self.call::<scan::PropsGet>(&())
    }
    pub fn scan_buffer_get(&mut self) -> NanonisTcpResult<scan::BufferGetResponse> {
        self.call::<scan::BufferGet>(&())
    }
    pub fn signals_names_get(&mut self) -> NanonisTcpResult<signals::NamesGetResponse> {
        self.call::<signals::NamesGet>(&())
    }
    pub fn scan_frame_data_grab(
        &mut self,
        channel_index: u32,
        data_dir: LineDir,
    ) -> NanonisTcpResult<scan::FrameDataGrabResponse> {
        self.call::<scan::FrameDataGrab>(&scan::FrameDataGrabArgs {
            channel_index,
            data_dir,
        })
    }
    pub fn call<C: Command>(&mut self, args: &C::Args) -> NanonisTcpResult<C::Response> {
        let fsm = NanonisTcpFsm::<C>::new(&mut self.buf, args)?;
        self.stream.write_all(fsm.bytes_to_write())?;
        self.stream.flush()?;
        let mut fsm = fsm.prepare_for_header()?;
        self.stream.read_exact(fsm.bytes_to_read_mut())?;
        let mut fsm = fsm.prepare_for_body()?;
        self.stream.read_exact(fsm.bytes_to_read_mut())?;
        fsm.parse_response()
    }
}
