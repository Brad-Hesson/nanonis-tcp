use macro_rules_attribute::apply;

use crate::{
    Command,
    codec::{CodecRead, CodecReadDerive, CodecWriteDerive, Vec2D},
};

// *********************
// * ScanBufferGet *
// *********************

pub struct ScanBufferGet;
impl Command for ScanBufferGet {
    const NAME: &'static str = "Scan.BufferGet";
    type Args = ();
    type Response = ScanBufferGetResponse;
}
#[derive(Debug)]
#[apply(CodecReadDerive)]
pub struct ScanBufferGetResponse {
    pub channel_indexes: Vec<i32>,
    pub px_per_line: i32,
    pub num_lines: i32,
}

// **********************
// *    ScanPropsGet    *
// **********************

pub struct ScanPropsGet;
impl Command for ScanPropsGet {
    const NAME: &'static str = "Scan.PropsGet";
    type Args = ();
    type Response = ScanPropsGetResponse;
}
#[apply(CodecReadDerive)]
#[derive(Debug)]
pub struct ScanPropsGetResponse {
    pub continuous_scan: u32,
    pub bouncy_scan: u32,
    pub autosave: u32,
    pub series_name: String,
    pub comment: String,
    _modules_names_size: i32,
    pub modules_names: Vec<String>,
    pub params_per_mod: Vec<i32>,
    pub parameters: Vec2D<String>,
}

// *********************
// * ScanFrameDataGrab *
// *********************

pub struct ScanFrameDataGrab;
impl Command for ScanFrameDataGrab {
    const NAME: &'static str = "Scan.FrameDataGrab";
    type Args = ScanFrameDataGrabArgs;
    type Response = ScanFrameDataGrabResponse;
}
#[apply(CodecWriteDerive)]
pub struct ScanFrameDataGrabArgs {
    pub channel_index: u32,
    pub data_dir: u32,
}
#[derive(Debug)]
pub struct ScanFrameDataGrabResponse {
    pub channel_name: String,
    pub scan_data: Vec2D<f32>,
    pub scan_dir: u32,
}
impl CodecRead for ScanFrameDataGrabResponse {
    fn codec_read(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        let channel_name = String::codec_read(reader)?;
        let scan_data = (!channel_name.is_empty())
            .then(|| <Vec2D<f32>>::codec_read(reader))
            .transpose()?
            .unwrap_or_default();
        let scan_dir = u32::codec_read(reader)?;
        Ok(Self {
            channel_name,
            scan_data,
            scan_dir,
        })
    }
}

// *********************
// *      BiasSet      *
// *********************

pub struct BiasSet;
impl Command for BiasSet {
    const NAME: &'static str = "Bias.Set";
    type Args = BiasSetReq;
    type Response = ();
}
#[apply(CodecWriteDerive)]
pub struct BiasSetReq {
    pub bias: f32,
}

// *********************
// *  SignalsNamesGet  *
// *********************

pub struct SignalsNamesGet;
impl Command for SignalsNamesGet {
    const NAME: &'static str = "Signals.NamesGet";
    type Args = ();
    type Response = SignalsNamesGetResponse;
}
#[derive(Debug)]
#[apply(CodecReadDerive)]
pub struct SignalsNamesGetResponse {
    _names_size: i32,
    pub names: Vec<String>,
}
