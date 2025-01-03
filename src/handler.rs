use std::{
  sync::Arc,
  time::Duration
};

use crossterm::event::{EventStream, Event};
use futures::{FutureExt, StreamExt};
use ratatui::layout::Size;
use tokio::sync::RwLock;

use opencv::{prelude::*, imgproc, videoio::{self, VideoCapture, VideoCaptureTrait}};

use crate::channel::AppEvent;
use crate::app::SCALE_FACTOR;
use crate::app::ASCII_CHARS;

type TerminalSize = (u16, u16);

#[derive(Eq, PartialEq)]
pub enum ImageConvertType {
  Colorful,
  GrayScale,
  Threshold
}

/// Frame handler config
pub struct FrameHandlerConfig {
  /// Image convert type (camera mode)
  pub image_convert_type: ImageConvertType,

  /// Terminal size (widht, height)
  pub terminal_size: TerminalSize
}

impl FrameHandlerConfig {
  pub fn new(terminal_size: Size) -> Self {
    Self {
      image_convert_type: ImageConvertType::GrayScale,
      terminal_size: (terminal_size.width, terminal_size.height)
    }
  }
}

/// Converts a frame into a grayscale
fn convert_into_grayscale(frame: &opencv::core::Mat, res_frame: &mut opencv::core::Mat) {
  imgproc::cvt_color(frame, res_frame, imgproc::COLOR_BGR2GRAY, 0).unwrap()
}

/// Converts a camera frame into ASCII
///
/// Resize the frame to a smaller size
///
/// Inserts an ASCII_CHAR based on the intensity
pub fn convert_frame_into_ascii(
  frame: opencv::core::Mat,
  image_convert_type: &ImageConvertType
) -> String {
  let mut ascii_image = String::new();

  for y in 0..frame.rows() {
    for x in 0..frame.cols() {
      let intensity = frame.at_2d::<u8>(y, x).unwrap();
      let ascii_char = if *image_convert_type == ImageConvertType::Threshold {
        if *intensity > 150 { '█' } else { ' ' }
      } else {
        let char_index = (*intensity as f32 * (ASCII_CHARS.len() - 1) as f32 / 255.0).round() as usize;
        ASCII_CHARS[char_index]
      };

      ascii_image.push(ascii_char);
    }

    ascii_image.push_str("\n");
  }

  ascii_image
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

        let mut small_frame = opencv::core::Mat::default();

        let config = config.read().await;

        let cam_size = opencv::core::Size {
          width: (config.terminal_size.0 / SCALE_FACTOR) as i32,
          height: (config.terminal_size.1 / SCALE_FACTOR) as i32
        };

        opencv::imgproc::resize(
          &frame,
          &mut small_frame,
          cam_size, 0.0, 0.0, opencv::imgproc::INTER_LINEAR
        ).unwrap();

        let res_frame = match config.image_convert_type {
          ImageConvertType::Colorful => small_frame.clone(),
          ImageConvertType::GrayScale => {
            let mut gray_frame = opencv::core::Mat::default();
            convert_into_grayscale(&small_frame, &mut gray_frame);
            gray_frame
          },
          ImageConvertType::Threshold => {
            let mut gray_frame = opencv::core::Mat::default();
            let mut binary_frame = opencv::core::Mat::default();

            convert_into_grayscale(&small_frame, &mut gray_frame);

            imgproc::threshold(
              &gray_frame,
              &mut binary_frame,
              128.0, 255.0,
              imgproc::THRESH_BINARY
            ).unwrap();

            binary_frame
          }
        };

        let ascii_frame = convert_frame_into_ascii(res_frame, &config.image_convert_type);

        if tx.send(AppEvent::AsciiFrame(ascii_frame)).is_err() {
          break;
        }

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
            Event::Resize(width, height) => tx.send(AppEvent::TerminalResize((width, height))).unwrap(),
            _ => {}
          }
        }
      }
    });

    Self(handle)
  }
}
