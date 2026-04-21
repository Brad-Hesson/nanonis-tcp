use macro_rules_attribute::apply;

use crate::{CodecReadDerive, CodecWriteDerive, commands::Command};

/// Returns the values of the X and Y signals
pub struct XYPosGet;
impl Command for XYPosGet {
    const NAME: &'static str = "FolMe.XYPosGet";
    type Args = XYPosGetArgs;
    type Response = XYPosGetResponse;
}
#[apply(CodecWriteDerive)]
pub struct XYPosGetArgs {
    /// selects whether the function returns the next available signal value
    // or if it waits for a full period of new data. If False, this function returns a value 0 to Tap seconds after being
    // called. If True, the function discards the first oversampled signal value received but returns the second
    // value received. Thus, the function returns a value Tap to 2*Tap seconds after being called.
    pub wait_newest: bool,
}
#[derive(Debug)]
#[apply(CodecReadDerive)]
pub struct XYPosGetResponse {
    pub x_pos: f64,
    pub y_pos: f64,
}