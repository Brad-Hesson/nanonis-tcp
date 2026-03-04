use std::fmt::Debug;

use binrw::{binread, binwrite};

use crate::Command;

// *********************
// * ScanBufferGet *
// *********************

pub struct ScanBufferGet;
#[binread]
#[derive(Debug)]
pub struct ScanBufferGetResp {
    #[br(temp)]
    channels_num: i32,
    #[br(count = channels_num)]
    pub channel_indexes: Vec<i32>,
    pub px_per_line: i32,
    pub num_lines: i32,
}
impl Command for ScanBufferGet {
    const NAME: &'static str = "Scan.BufferGet";
    type Request = ();
    type Response = ScanBufferGetResp;
}

// *********************
// * ScanPropsGet *
// *********************

pub struct ScanPropsGet;
#[binread]
#[derive(Debug)]
pub struct ScanPropsGetResp {
    pub continuous_scan: u32,
    pub bouncy_scan: u32,
    pub autosave: u32,
    #[br(map = |p: PrefixString| p.inner)]
    pub series_name: String,
    #[br(map = |p: PrefixString| p.inner)]
    pub comment: String,
    
    #[br(temp)]
    pub modules_names_size: i32,
    #[br(temp)]
    pub modules_names_number: i32,
    #[br(count = modules_names_number, map = |s: Vec<PrefixString>| s.into_iter().map(|s| s.inner).collect())]
    pub modules_names: Vec<String>,
    
    #[br(temp)]
    pub params_per_mod_len: i32,
    #[br(count = params_per_mod_len)]
    #[br(dbg)]
    pub params_per_mod: Vec<i32>,
    
    #[br(temp)]
    #[br(dbg)]
    pub parameters_rows: i32,
    #[br(temp, if(parameters_rows > 0))]
    #[br(dbg)]
    pub parameters_columns: i32,
    #[br(dbg)]
    #[br(if(parameters_rows > 0), count = parameters_rows*parameters_columns + 2, map = |s: Vec<PrefixString>| s.into_iter().map(|s| s.inner).collect())]
    pub parameters: Vec<String>,
    
    #[br(dbg)]
    pub autopaste: u32,
}
impl Command for ScanPropsGet {
    const NAME: &'static str = "Scan.PropsGet";
    type Request = ();
    type Response = ScanPropsGetResp;
}

// *********************
// * ScanFrameDataGrab *
// *********************

pub struct ScanFrameDataGrab;
#[binwrite]
pub struct ScanFrameDataGrabReq {
    pub channel_index: u32,
    pub data_dir: u32,
}
#[binread]
#[derive(Debug)]
pub struct ScanFrameDataGrabResp {
    #[br(temp)]
    pub name_len: i32,
    #[br(count = name_len, map = |b: Vec<u8>| String::from_utf8_lossy(&b).to_string())]
    pub channel_name: String,
    pub scan_rows: i32,
    #[br(if(scan_rows > 0))]
    pub scan_cols: i32,
    #[br(if(scan_rows > 0))]
    #[br(count = scan_rows * scan_cols)]
    pub scan_data: Vec<f32>,
    #[br(if(scan_rows > 0))]
    pub scan_dir: u32,
}
impl Command for ScanFrameDataGrab {
    const NAME: &'static str = "Scan.FrameDataGrab";
    type Request = ScanFrameDataGrabReq;
    type Response = ScanFrameDataGrabResp;
}

// *********************
// *      BiasSet      *
// *********************

pub struct BiasSet;
#[binwrite]
pub struct BiasSetReq {
    pub bias: f32,
}
impl Command for BiasSet {
    const NAME: &'static str = "Bias.Set";
    type Request = BiasSetReq;
    type Response = ();
}

// *********************
// *  SignalsNamesGet  *
// *********************

pub struct SignalsNamesGet;
#[binread]
#[derive(Debug)]
pub struct SignalsNamesGetResp {
    #[br(temp)]
    pub names_size: i32,
    #[br(temp)]
    pub names_num: i32,
    #[br(count = names_num, map = |s: Vec<PrefixString>| s.into_iter().map(|s| s.inner).collect())]
    pub names: Vec<String>,
}
impl Command for SignalsNamesGet {
    const NAME: &'static str = "Signals.NamesGet";
    type Request = ();
    type Response = SignalsNamesGetResp;
}

#[binread]
#[derive(Debug)]
pub struct PrefixString {
    #[br(temp)]
    pub len: i32,
    #[br(count = len, map = |b: Vec<u8>| String::from_utf8_lossy(&b).to_string())]
    pub inner: String,
}
