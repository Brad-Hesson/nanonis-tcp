use std::time::Duration;

use crate::codec::{CodecRead, CodecWrite};

pub mod blocking;
mod codec;
pub mod commands;
pub mod error;
pub mod fsm;
#[cfg(feature = "tokio")]
pub mod nonblocking;

#[derive(Debug, Clone, Copy, num_enum::IntoPrimitive)]
#[repr(u16)]
pub enum ActionType {
    Start = 0,
    Stop = 1,
    Pause = 2,
    Resume = 3,
    Freeze = 4,
    Unfreeze = 5,
    GoToCenter = 6,
}
impl CodecWrite for ActionType {
    fn codec_write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        u16::from(*self).codec_write(writer)
    }

    fn codec_len(&self) -> usize {
        size_of::<u16>()
    }
}

#[derive(Debug, Clone, Copy, num_enum::IntoPrimitive)]
#[repr(u32)]
pub enum ScanDir {
    Down = 0,
    Up = 1,
}
impl CodecWrite for ScanDir {
    fn codec_write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        u32::from(*self).codec_write(writer)
    }

    fn codec_len(&self) -> usize {
        size_of::<u32>()
    }
}

#[derive(
    Debug, Clone, Copy, num_enum::IntoPrimitive, num_enum::TryFromPrimitive, PartialEq, Eq,
)]
#[repr(u16)]
pub enum ScanMovementType {
    Forward = 0,
    Backward = 1,
    FrameCenter = 2,
    StartOfScan = 3,
}
impl CodecWrite for ScanMovementType {
    fn codec_write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        u16::from(*self).codec_write(writer)
    }

    fn codec_len(&self) -> usize {
        size_of::<u16>()
    }
}
impl CodecRead for ScanMovementType {
    fn codec_read(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        u16::codec_read(reader).map(|v| Self::try_from(v).unwrap())
    }
}

impl CodecWrite for Option<Duration> {
    fn codec_write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.map(|v| v.as_millis() as i32)
            .unwrap_or(-1)
            .codec_write(writer)
    }
    fn codec_len(&self) -> usize {
        size_of::<i32>()
    }
}

impl CodecRead for bool {
    fn codec_read(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        u32::codec_read(reader).map(|v| v != 0)
    }
}
impl CodecWrite for bool {
    fn codec_write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        (*self as u32).codec_write(writer)
    }

    fn codec_len(&self) -> usize {
        size_of::<u32>()
    }
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

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
        nanonis
            .scan_action(ActionType::Start, ScanDir::Down)
            .await
            .unwrap();
        while nanonis
            .scan_wait_end_of_line(Some(Duration::from_secs(1)))
            .await
            .unwrap()
            .movement_type
            != ScanMovementType::StartOfScan
        {}
        loop {
            let line_status = dbg!(
                nanonis
                    .scan_wait_end_of_line(Some(Duration::from_secs(1)))
                    .await
                    .unwrap()
            );
            if line_status.timed_out {
                break;
            }
            // let data = nanonis.scan_frame_data_grab(30, 0).await.unwrap();
            // let width = data.scan_data.size[0] as usize;
            // let line_num = line_status.line_number as usize - 1;
            // let line = &data.scan_data.data[255..][..width];
            // println!("{:?}", line);
        }
    }
}
