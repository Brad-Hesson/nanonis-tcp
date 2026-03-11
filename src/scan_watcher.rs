use std::marker::PhantomData;

use tokio::{
    net::ToSocketAddrs,
    sync::{
        mpsc::{Receiver, Sender},
        mpsc::{UnboundedReceiver, UnboundedSender},
    },
};

use crate::{
    LineDir, ScanMovementType, commands::scan::FrameDataGrabResponse, error::NanonisTcpResult,
    nonblocking::NanonisTcp,
};

pub struct ScanWatcher<C: Callback> {
    _phantom: PhantomData<C>,
    close_tx: Sender<()>,
    channel_tx: tokio::sync::watch::Sender<u32>,
    line_dir_tx: tokio::sync::watch::Sender<LineDir>,
}
impl<C: Callback + 'static> ScanWatcher<C> {
    pub async fn new(
        addr1: impl ToSocketAddrs,
        addr2: impl ToSocketAddrs,
        callback: C,
    ) -> NanonisTcpResult<Self> {
        let line_tcp = NanonisTcp::new(addr1).await?;
        let frame_tcp = NanonisTcp::new(addr2).await?;
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
        let (close_tx, close_rx) = tokio::sync::mpsc::channel::<()>(1);
        let (channel_tx, channel_rx) = tokio::sync::watch::channel(0);
        let (line_dir_tx, line_dir_rx) = tokio::sync::watch::channel(LineDir::Forward);
        tokio::spawn(line_worker(line_tcp, event_tx, close_rx));
        tokio::spawn(frame_worker(
            frame_tcp,
            event_rx,
            channel_rx,
            line_dir_rx,
            callback,
        ));
        Ok(Self {
            _phantom: PhantomData,
            close_tx,
            channel_tx,
            line_dir_tx,
        })
    }
    pub fn set_channel(&mut self, channel: u32) {
        self.channel_tx.send(channel).unwrap()
    }
    pub fn set_line_dir(&mut self, line_dir: LineDir) {
        self.line_dir_tx.send(line_dir).unwrap()
    }
}
impl<C: Callback> Drop for ScanWatcher<C> {
    fn drop(&mut self) {
        self.close_tx.blocking_send(()).ok();
    }
}

fn line_worker(
    mut line_tcp: NanonisTcp,
    event_tx: UnboundedSender<LineEvent>,
    close_rx: Receiver<()>,
) -> impl Future<Output = ()> {
    async move {
        while close_rx.is_empty() {
            let line = line_tcp.scan_wait_end_of_line(None).await.unwrap();
            let event = match line.movement_type {
                ScanMovementType::Scan(line_dir) => LineEvent::Line {
                    line_number: line.line_number as usize - 1,
                    line_dir,
                },
                ScanMovementType::StartOfScan => LineEvent::Start,
                _ => continue,
            };
            if event_tx.send(event).is_err() {
                break;
            }
        }
    }
}

fn frame_worker<C: Callback>(
    mut frame_tcp: NanonisTcp,
    event_rx: UnboundedReceiver<LineEvent>,
    channel_rx: tokio::sync::watch::Receiver<u32>,
    line_dir_rx: tokio::sync::watch::Receiver<LineDir>,
    mut callback: C,
) -> impl Future<Output = ()> {
    async move {
        let mut event_rx = EagerPeek::new(event_rx);
        while let Some(mut event) = event_rx.recv().await {
            if matches!(event, LineEvent::Line { .. }) {
                while matches!(event_rx.peek_eager(), Some(LineEvent::Line { .. })) {
                    event = event_rx.recv().await.unwrap();
                }
            }
            match event {
                LineEvent::Line {
                    line_number,
                    line_dir,
                } => {
                    let requested_line_dir = *line_dir_rx.borrow();
                    if requested_line_dir == line_dir {
                        let channel = *channel_rx.borrow();
                        let frame = frame_tcp
                            .scan_frame_data_grab(channel, line_dir)
                            .await
                            .unwrap();
                        callback.frame(line_number, frame);
                    }
                }
                LineEvent::Start => callback.start(),
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LineEvent {
    Line {
        line_number: usize,
        line_dir: LineDir,
    },
    Start,
}

struct EagerPeek<T> {
    buffered: Option<T>,
    rx: UnboundedReceiver<T>,
}
impl<T> EagerPeek<T> {
    fn new(rx: UnboundedReceiver<T>) -> Self {
        Self { buffered: None, rx }
    }
    fn peek_eager(&mut self) -> Option<&T> {
        if self.buffered.is_none() {
            self.buffered = self.rx.try_recv().ok();
        }
        self.buffered.as_ref()
    }
    async fn recv(&mut self) -> Option<T> {
        match self.buffered.take() {
            Some(v) => Some(v),
            None => self.rx.recv().await,
        }
    }
}

pub trait Callback: Send {
    fn frame(&mut self, num_lines: usize, frame: FrameDataGrabResponse);
    fn start(&mut self);
}
