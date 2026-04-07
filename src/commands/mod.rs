use crate::codec::{CodecRead, CodecWrite};

pub mod bias;
pub mod scan;
pub mod signals;
pub mod piezo;

pub trait Command {
    const NAME: &'static str;
    type Args: CodecWrite;
    type Response: CodecRead;
}
