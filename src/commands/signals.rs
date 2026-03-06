use macro_rules_attribute::apply;

use crate::{codec::CodecReadDerive, commands::Command};

// *********************
// *  NamesGet  *
// *********************
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
    pub names: Vec<String>,
}
