use std::time::Duration;

use crossterm::event::{EventStream, Event};
use futures::{FutureExt, StreamExt};

use opencv::{imgproc, videoio::{self, VideoCapture, VideoCaptureTrait}};

use crate::channel::AppEvent;

#[allow(unused)]
pub struct FrameHandler(pub tokio::task::JoinHandle<()>);

impl FrameHandler {
  pub fn try_new(tx: tokio::sync::mpsc::UnboundedSender<AppEvent>) -> opencv::Result<Self> {
    let mut cam = VideoCapture::new(0, videoio::CAP_ANY)?;
    let mut frame = opencv::core::Mat::default();

    let handle = tokio::spawn(async move {
      let mut interval = tokio::time::interval(Duration::from_millis(50));

      loop {
        cam.read(&mut frame).unwrap();

        let mut gray_frame = opencv::core::Mat::default();
        imgproc::cvt_color(&frame, &mut gray_frame, imgproc::COLOR_BGR2GRAY, 0).unwrap();

        tx.send(AppEvent::Frame(gray_frame)).unwrap();
        interval.tick().await;
      }
    });

    Ok(Self(handle))
  }
}

#[allow(unused)]
pub struct EventHandler(pub tokio::task::JoinHandle<()>);

impl EventHandler {
  pub fn new(tx: tokio::sync::mpsc::UnboundedSender<AppEvent>) -> Self {
    let handle = tokio::spawn(async move {
      let mut reader = EventStream::new();

      loop {
        let crossterm_event = reader.next().fuse().await;

        if let Some(Ok(event)) = crossterm_event {
          match event {
            Event::Key(key_code) => tx.send(AppEvent::Event(key_code)).unwrap(),
            _ => {}
          }
        }
      }
    });

    Self(handle)
  }
}
