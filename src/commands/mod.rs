use crate::codec::{CodecRead, CodecWrite};

pub mod bias;
pub mod piezo;
pub mod scan;
pub mod signals;

#[allow(private_bounds)]
pub trait Command {
    const NAME: &'static str;
    type Args: CodecWrite;
    type Response: CodecRead;
}
