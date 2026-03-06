use std::time::Duration;

use macro_rules_attribute::apply;

use crate::{
    ActionType, ScanDir, ScanMovementType,
    codec::{CodecRead, CodecReadDerive, CodecWriteDerive, Vec2D},
    commands::Command,
};

/// Starts, stops, pauses or resumes a scan.
pub struct Action;
impl Command for Action {
    const NAME: &'static str = "Scan.Action";
    type Args = ActionArgs;
    type Response = ();
}
#[derive(Debug)]
#[apply(CodecWriteDerive)]
pub struct ActionArgs {
    /// sets which action to perform, where 0=Start, 1=Stop, 2=Pause, 3=Resume,
    /// 4=Freeze, 5=Unfreeze, 6=Go to Center
    pub action: ActionType,
    /// if 1, scan direction is set to up. If 0, direction is down
    pub dir: ScanDir,
}

/// Returns if the scan is running or not.
pub struct StatusGet;
impl Command for StatusGet {
    const NAME: &'static str = "Scan.StatusGet";
    type Args = ();
    type Response = StatusGetResponse;
}
#[derive(Debug)]
#[apply(CodecReadDerive)]
pub struct StatusGetResponse {
    /// means that if it is 1, scan is running. If 0, scan is not running
    pub running: bool,
}

/// Waits for the End-of-Scan.
/// This function returns only when an End-of-Line or timeout occurs (whichever occurs first).
pub struct WaitEndOfLine;
impl Command for WaitEndOfLine {
    const NAME: &'static str = "Scan.WaitEndOfLine";
    type Args = WaitEndOfLineArgs;
    type Response = WaitEndOfLineResponse;
}
#[apply(CodecWriteDerive)]
pub struct WaitEndOfLineArgs {
    /// sets how many milliseconds this function waits for an End-of-Scan. If –1, it waits indefinitely
    pub timeout: Option<Duration>,
}
#[derive(Debug)]
#[apply(CodecReadDerive)]
pub struct WaitEndOfLineResponse {
    /// means that if it is 1, the function timed-out. If 0, it didn’t time-out
    pub timed_out: bool,
    /// the line number of the last completed line
    pub line_number: i32,
    /// can be forward (0) or backward (1) while scanning, moved to the scan
    /// frame center (2), or moved to start point of the scan frame right before starting to scan (3)
    pub movement_type: ScanMovementType,
    /// the pass number of the last completed line (relevant when MultiPass is enabled)
    pub pass_number: i32,
}

/// Returns the scan frame parameters.
pub struct FrameGet;
impl Command for FrameGet {
    const NAME: &'static str = "Scan.FrameGet";
    type Args = ();
    type Response = FrameGetResponse;
}
#[derive(Debug)]
#[apply(CodecReadDerive)]
pub struct FrameGetResponse {
    /// the X position of the scan frame center
    pub center_x: f32,
    /// the Y position of the scan frame center
    pub center_y: f32,
    /// the width of the scan frame
    pub width: f32,
    /// the height of the scan frame
    pub height: f32,
    /// the angle of the scan frame (positive angle means clockwise rotation)
    pub angle: f32,
}

/// Returns the scan buffer parameters.
pub struct BufferGet;
impl Command for BufferGet {
    const NAME: &'static str = "Scan.BufferGet";
    type Args = ();
    type Response = BufferGetResponse;
}
#[derive(Debug)]
#[apply(CodecReadDerive)]
pub struct BufferGetResponse {
    /// the indexes of recorded channels. The index is comprised between 0
    /// and 127, and it corresponds to the full list of signals available in the system.
    /// To get the signal name and its corresponding index in the list of the 128 available signals in the Nanonis
    /// Controller, use the Signal.NamesGet function, or check the RT Idx value in the Signals Manager module.
    pub channel_indexes: Vec<i32>,
    /// the number of pixels per line
    pub px_per_line: i32,
    /// the number of scan lines
    pub num_lines: i32,
}

/// Returns some of the scan parameters.
pub struct PropsGet;
impl Command for PropsGet {
    const NAME: &'static str = "Scan.PropsGet";
    type Args = ();
    type Response = PropsGetResponse;
}
#[apply(CodecReadDerive)]
#[derive(Debug)]
pub struct PropsGetResponse {
    ///  indicates whether the scan continues or stops when a frame has been
    /// completed. 0 means Off, and 1 is On
    pub continuous_scan: bool,
    /// indicates whether the scan direction changes when a frame has been
    /// completed. 0 means Off, and 1 is On
    pub bouncy_scan: bool,
    /// defines the save behavior when a frame has been completed. "All" saves all the
    /// future images. "Next" only saves the next frame. 0 is All, 1 is Next, and 2 means Off
    pub autosave: u32,
    /// base name used for the saved images
    pub series_name: String,
    /// comment saved in the file
    pub comment: String,
    _modules_names_size: i32,
    /// an array of modules names strings
    pub modules_names: Vec<String>,
    /// an array containing the number of parameters per module
    pub params_per_mod: Vec<i32>,
    /// returns the parameters that are going to be saved in the header of the image
    /// files. Each row of parameters belongs to a different module.
    pub parameters: Vec2D<String>,
}

/// Returns the scan data of the selected frame.
pub struct FrameDataGrab;
impl Command for FrameDataGrab {
    const NAME: &'static str = "Scan.FrameDataGrab";
    type Args = FrameDataGrabArgs;
    type Response = FrameDataGrabResponse;
}
#[apply(CodecWriteDerive)]
pub struct FrameDataGrabArgs {
    /// Selects which channel to get the data from.
    /// The channel must be one of the acquired channels.
    /// The list of acquired channels while scanning can be configured by the function Scan.BufferSet.
    /// The index is comprised between 0 and 127, and it corresponds to the full list of signals available in the
    /// system.
    /// To get the signal name and its corresponding index in the list of the 128 available signals in the Nanonis
    /// Controller, use the Signal.NamesGet function, or check the RT Idx value in the Signals Manager module.
    pub channel_index: u32,
    /// Selects the data direction, where 1 is forward, and 0 is backward
    pub data_dir: u32,
}
#[derive(Debug)]
pub struct FrameDataGrabResponse {
    /// the name of the channel selected by Channel index
    pub channel_name: String,
    /// the scan frame data of the selected channel
    pub scan_data: Vec2D<f32>,
    /// the scan direction, where 1 is up, and 0 is down
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
