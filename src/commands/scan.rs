// *********************
// * BufferGet *
// *********************

use macro_rules_attribute::apply;

use crate::{
    codec::{CodecRead, CodecReadDerive, CodecWriteDerive, Vec2D},
    commands::Command,
};

pub struct BufferGet;
impl Command for BufferGet {
    const NAME: &'static str = "Scan.BufferGet";
    type Args = ();
    type Response = BufferGetResponse;
}
#[derive(Debug)]
#[apply(CodecReadDerive)]
pub struct BufferGetResponse {
    pub channel_indexes: Vec<i32>,
    pub px_per_line: i32,
    pub num_lines: i32,
}

// **********************
// *    PropsGet    *
// **********************

pub struct PropsGet;
impl Command for PropsGet {
    const NAME: &'static str = "Scan.PropsGet";
    type Args = ();
    type Response = PropsGetResponse;
}
#[apply(CodecReadDerive)]
#[derive(Debug)]
pub struct PropsGetResponse {
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
// * FrameDataGrab *
// *********************

pub struct FrameDataGrab;
impl Command for FrameDataGrab {
    const NAME: &'static str = "Scan.FrameDataGrab";
    type Args = FrameDataGrabArgs;
    type Response = FrameDataGrabResponse;
}
#[apply(CodecWriteDerive)]
pub struct FrameDataGrabArgs {
    pub channel_index: u32,
    pub data_dir: u32,
}
#[derive(Debug)]
pub struct FrameDataGrabResponse {
    pub channel_name: String,
    pub scan_data: Vec2D<f32>,
    pub scan_dir: u32,
}
impl CodecRead for FrameDataGrabResponse {
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
