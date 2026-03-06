use macro_rules_attribute::apply;

use crate::{commands::Command, codec::CodecWriteDerive};

// *********************
// *      Set      *
// *********************
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
