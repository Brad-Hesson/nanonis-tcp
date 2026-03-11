use std::time::Duration;

use crate::{
    codec::{CodecRead, CodecWrite},
    commands::scan,
};

pub mod blocking;
mod codec;
pub mod commands;
pub mod error;
pub mod fsm;
#[cfg(feature = "tokio")]
pub mod nonblocking;
#[cfg(feature = "tokio")]
pub mod scan_watcher;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanMovementType {
    Scan(LineDirection),
    FrameCenter,
    StartOfScan,
}
impl CodecWrite for ScanMovementType {
    fn codec_write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        match self {
            ScanMovementType::Scan(LineDirection::Forward) => 0u16,
            ScanMovementType::Scan(LineDirection::Backward) => 1,
            ScanMovementType::FrameCenter => 2,
            ScanMovementType::StartOfScan => 3,
        }
        .codec_write(writer)
    }

    fn codec_len(&self) -> usize {
        size_of::<u16>()
    }
}
impl CodecRead for ScanMovementType {
    fn codec_read(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        match u16::codec_read(reader)? {
            0 => Ok(Self::Scan(LineDirection::Forward)),
            1 => Ok(Self::Scan(LineDirection::Backward)),
            2 => Ok(Self::FrameCenter),
            3 => Ok(Self::StartOfScan),
            _ => unreachable!(),
        }
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
impl CodecRead for usize {
    fn codec_read(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        i32::codec_read(reader)?
            .try_into()
            .map_err(std::io::Error::other)
    }
}
impl CodecWrite for usize {
    fn codec_write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        (*self as i32).codec_write(writer)
    }

    fn codec_len(&self) -> usize {
        size_of::<i32>()
    }
}

#[cfg(test)]
mod tests {
    use crate::scan_watcher::ScanWatcher;

    use super::*;

    #[test]
    fn blocking() {
        let mut nanonis = blocking::NanonisTcp::new("localhost:6501").unwrap();
        let frame = nanonis.scan_frame_get().unwrap();
        println!("{frame:?}")
    }

    #[tokio::test]
    async fn asink() {
        console_subscriber::init();
        let sw = ScanWatcher::new("localhost:6501", "localhost:6502", PrintCallback)
            .await
            .unwrap();
        loop {
            tokio::task::yield_now().await
        }
    }
}

struct PrintCallback;
impl scan_watcher::Callback for PrintCallback {
    fn frame(&mut self, line_number: usize, frame: scan::FrameDataGrabResponse) {
        println!("frame: {line_number:?} {}", frame.scan_dir);
    }

    fn start(&mut self) {
        println!("start")
    }
}

#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, num_enum::IntoPrimitive)]
pub enum LineDirection {
    Forward = 1,
    Backward = 0,
}
