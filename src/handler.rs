use std::time::Duration;

use crossterm::event::{EventStream, Event};
use ratatui::layout::Size;
use futures::{FutureExt, StreamExt};

use opencv::{prelude::*, imgproc, videoio::{self, VideoCapture, VideoCaptureTrait}};

use crate::{app::SCALE_FACTOR, channel::AppEvent};

const ASCII_CHARS: &[u8] = b"@%#*+=-:. ";

/// Converts an image to ascii code
fn image_to_ascii(frame: &opencv::core::Mat) -> String {
  let mut result = String::new();

  for y in 0..frame.rows() {
    for x in 0..frame.cols() {
      let intensity = frame.at_2d::<u8>(y, x).unwrap();
      let char_index = (*intensity as f32 * (ASCII_CHARS.len() - 1) as f32 / 255.0).round() as usize;
      let ascii_char = ASCII_CHARS[char_index] as char;

      result.push(ascii_char);
    }

    result.push_str("\n");
  }

  result
}

pub struct FrameHandler(pub tokio::task::JoinHandle<()>);

impl FrameHandler {
  pub fn try_new(terminal_size: Size, tx: tokio::sync::mpsc::UnboundedSender<AppEvent>) -> opencv::Result<Self> {
    let mut cam = VideoCapture::new(0, videoio::CAP_ANY)?;
    let mut frame = opencv::core::Mat::default();
    let mut gray_frame = opencv::core::Mat::default();
    let mut small_frame = opencv::core::Mat::default(); let image_size = opencv::core::Size {
      width: (terminal_size.width / SCALE_FACTOR) as i32,
      height: (terminal_size.height / SCALE_FACTOR) as i32
    };

    let handle = tokio::spawn(async move {
      let mut interval = tokio::time::interval(Duration::from_millis(300));

      loop {
        cam.read(&mut frame).unwrap();

        imgproc::cvt_color(&frame, &mut gray_frame, imgproc::COLOR_BGR2GRAY, 0).unwrap();

        imgproc::resize(
          &gray_frame,
          &mut small_frame,
          image_size, 0.0, 0.0, imgproc::INTER_LINEAR
        ).unwrap();

        let image_ascii = image_to_ascii(&small_frame);
        tx.send(AppEvent::Frame(image_ascii)).unwrap();
        interval.tick().await;
      }
    });

    Ok(Self(handle))
  }
}

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
