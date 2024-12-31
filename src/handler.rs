use std::{
  sync::{Arc, RwLock},
  time::Duration
};

use crossterm::event::{EventStream, Event};
use futures::{FutureExt, StreamExt};

use opencv::{imgproc, videoio::{self, VideoCapture, VideoCaptureTrait}};

use crate::channel::AppEvent;

#[derive(Eq, PartialEq)]
pub enum ImageConvertType {
  GrayScale,
  Threshold
}

/// Frame handler config
pub struct FrameHandlerConfig {
  /// Image convert type (camera mode)
  pub image_convert_type: ImageConvertType
}

impl Default for FrameHandlerConfig {
  fn default() -> Self {
    Self {
      image_convert_type: ImageConvertType::GrayScale
    }
  }
}

#[allow(unused)]
pub struct FrameHandler;

impl FrameHandler {
  /// Spawns a new tokio task
  ///
  /// Opens a device camera
  ///
  /// Converts image into a grayscale image
  pub fn try_new(
    config: Arc<RwLock<FrameHandlerConfig>>,
    tx: tokio::sync::mpsc::UnboundedSender<AppEvent>
  ) -> opencv::Result<Self> {
    let mut cam = VideoCapture::new(0, videoio::CAP_ANY)?;
    let mut frame = opencv::core::Mat::default();

    let _handle = tokio::spawn(async move {
      // Camera frame delay
      let mut interval = tokio::time::interval(Duration::from_millis(50));

      loop {
        cam.read(&mut frame).unwrap();

        let mut gray_frame = opencv::core::Mat::default();

        imgproc::cvt_color(&frame, &mut gray_frame, imgproc::COLOR_BGR2GRAY, 0).unwrap();

        let res_frame = if config.read().unwrap().image_convert_type == ImageConvertType::Threshold {
          let mut binary_frame = opencv::core::Mat::default();

          imgproc::threshold(
            &gray_frame,
            &mut binary_frame,
            128.0, 255.0,
            imgproc::THRESH_BINARY
          ).unwrap();

          binary_frame
        } else {
          gray_frame
        };

        tx.send(AppEvent::Frame(res_frame)).unwrap();
        interval.tick().await;
      }
    });

    Ok(Self {})
  }
}

#[allow(unused)]
pub struct EventHandler(pub tokio::task::JoinHandle<()>);

impl EventHandler {
  /// Spawns a new tokio task
  ///
  /// Wait on a crossterm event
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
