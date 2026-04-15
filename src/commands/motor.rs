use macro_rules_attribute::apply;

use crate::{CodecWriteDerive, MotorDir, commands::Command};

/// Moves the coarse positioning device (motor, piezo actuator...).
pub struct StartMove;
impl Command for StartMove {
    const NAME: &'static str = "Motor.StartMove";
    type Args = StartMoveArgs;
    type Response = ();
}
#[derive(Debug)]
#[apply(CodecWriteDerive)]
pub struct StartMoveArgs {
    /// selects in which direction to move. Note that depending on your motor
    /// controller and setup only the Z axis or even only Z- may work.
    pub dir: MotorDir,
    /// defines the number of steps to move in the specified direction
    pub num_steps: u16,
    /// the selection of the groups defined in the motor control module. If the motor
    /// doesn’t support the selection of groups, set it to 0.  
    /// Valid values are 0=Group 1, 1=Group 2, 2=Group 3, 3=Group 4, 4=Group 5, 5=Group 6
    pub group: u32,
    /// defines if this function only returns when the motor reaches
    /// its destination
    pub blocking: bool,
}
