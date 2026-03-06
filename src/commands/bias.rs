use macro_rules_attribute::apply;

use crate::{codec::CodecWriteDerive, commands::Command};

/// Sets the Bias voltage to the specified value.
pub struct Set;
impl Command for Set {
    const NAME: &'static str = "Bias.Set";
    type Args = SetArgs;
    type Response = ();
}
#[apply(CodecWriteDerive)]
pub struct SetArgs {
    pub bias: f32,
}
