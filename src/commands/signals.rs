use macro_rules_attribute::apply;

use crate::{codec::CodecReadDerive, commands::Command};

/// Returns the signals names list of the 128 signals available in the software.
/// The 128 signals are physical inputs, physical outputs and internal channels. By searching in the list the channel’s
/// name you are interested in, you can get its index (0-127).
pub struct NamesGet;
impl Command for NamesGet {
    const NAME: &'static str = "Signals.NamesGet";
    type Args = ();
    type Response = NamesGetResponse;
}
#[derive(Debug)]
#[apply(CodecReadDerive)]
pub struct NamesGetResponse {
    _names_size: i32,
    /// an array of signals names strings
    pub names: Vec<String>,
}
