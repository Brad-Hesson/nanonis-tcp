use std::time::Duration;

use macro_rules_attribute::apply;

use crate::{CodecWriteDerive, commands::Command};

/// Withdraws the tip. 
/// This function switches off the Z-Controller and then fully withdraws the tip (to the upper limit of the Z-piezo range).
pub struct Withdraw;
impl Command for Withdraw {
    const NAME: &'static str = "ZCtrl.Withdraw";
    type Args = WithdrawArgs;
    type Response = ();
}
#[derive(Debug)]
#[apply(CodecWriteDerive)]
pub struct WithdrawArgs {
    pub wait_finished: bool,
    pub timeout: Option<Duration>
}

/// Switches the Z-Controller On or Off.
pub struct OnOffSet;
impl Command for OnOffSet {
    const NAME: &'static str = "ZCtrl.OnOffSet";
    type Args = OnOffSetArgs;
    type Response = ();
}
#[derive(Debug)]
#[apply(CodecWriteDerive)]
pub struct OnOffSetArgs {
    pub status: bool,
}
