use crate::codec::{CodecRead, CodecWrite};

pub mod blocking;
mod codec;
pub mod commands;
pub mod error;
pub mod fsm;
#[cfg(feature = "tokio")]
pub mod nonblocking;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn blocking() {
        let mut nanonis = blocking::NanonisTcp::new("glacier:6501").unwrap();
        let frame = nanonis.scan_frame_get().unwrap();
        println!("{frame:?}")
    }

    #[tokio::test]
    async fn asink() {
        let mut nanonis = nonblocking::NanonisTcp::new("glacier:6502").await.unwrap();
        nanonis.scan_action(0, 1).await.unwrap();
        while nanonis
            .scan_wait_end_of_line(1000)
            .await
            .unwrap()
            .movement_type
            != 3
        {}
        loop {
            let line_status = dbg!(nanonis.scan_wait_end_of_line(1000).await.unwrap());
            if line_status.timeout_status == 1 {
                break;
            }
            let data = nanonis.scan_frame_data_grab(30, 0).await.unwrap();
            let width = data.scan_data.size[0] as usize;
            let line_num = line_status.line_number as usize - 1;
            let line = &data.scan_data.data[255..][..width];
            println!("{:?}", line);
        }
    }
}
