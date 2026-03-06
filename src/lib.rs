use crate::codec::{CodecRead, CodecWrite};

pub mod blocking;
mod codec;
pub mod commands;
pub mod error;
pub mod fsm;
pub mod nonblocking;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn blocking() {
        let mut nanonis = blocking::NanonisTcp::new("glacier:6502");
        let props = nanonis.scan_props_get().unwrap();
        nanonis.scan_frame_data_grab(0, 0).ok();
        nanonis.scan_frame_data_grab(1, 0).ok();
        println!("{props:?}")
    }

    #[tokio::test]
    async fn asink() {
        let mut nanonis = nonblocking::NanonisTcp::new("glacier:6502").await.unwrap();
        let props = nanonis.scan_props_get().await;
        nanonis.scan_frame_data_grab(0, 0).await.ok();
        nanonis.scan_frame_data_grab(1, 0).await.ok();
        println!("{props:?}")
    }
}
