use std::{
  sync::Arc,
  time::Duration
};

use crossterm::event::{Event, EventStream};
use futures::{FutureExt, StreamExt};
use ratatui::{layout::Size, style::{Color, Style}, text::{Line, Span, Text}};
use tokio::sync::RwLock;

use opencv::{prelude::*, imgproc, videoio::{self, VideoCapture, VideoCaptureTrait}};

use crate::channel::AppEvent;
use crate::app::ASCII_CHARS;

type TerminalSize = (u16, u16);

#[derive(Eq, PartialEq, Clone)]
#[allow(unused)]
pub enum ImageConvertType {
  Colorful,
  GrayScale,
  Threshold
}

/// Camera window frame scale
#[derive(Clone)]
pub enum CamWindowScale {
  Full = 1,
  Small = 2,
}

/// Frame handler config
pub struct FrameHandlerConfig {
  /// Image convert type (camera mode)
  pub image_convert_type: ImageConvertType,

  /// Terminal size (width, height)
  pub terminal_size: TerminalSize,

  /// 
  pub cam_window_scale: CamWindowScale
}

impl FrameHandlerConfig {
  pub fn new(terminal_size: Size) -> Self {
    Self {
      image_convert_type: ImageConvertType::Colorful,
      terminal_size: (terminal_size.width, terminal_size.height),
      cam_window_scale: CamWindowScale::Small
    }
  }
}

/// Converts a frame into a grayscale.
fn convert_into_grayscale(frame: &opencv::core::Mat, res_frame: &mut opencv::core::Mat) {
  imgproc::cvt_color(frame, res_frame, imgproc::COLOR_BGR2GRAY, 0).unwrap()
}

/// Converts a camera frame into ASCII frame.
///
/// This method resizes the frame to a smaller size and then converts each pixel
/// into an ASCII character based on its intensity. The intensity is calculated
/// from the pixel's RGB values (Colorful), and the corresponding ASCII character is inserted
/// based on that intensity.
pub fn convert_frame_into_ascii(
  frame: opencv::core::Mat,
  image_convert_type: ImageConvertType
) -> Text<'static> {
  let mut lines = Vec::new();

  for y in 0..frame.rows() {
    let mut spans = Vec::new();
    for x in 0..frame.cols() {
      let (ascii_char, color) = match image_convert_type {
        ImageConvertType::Colorful => {
          let pixel = frame.at_2d::<opencv::core::Vec3b>(y, x).unwrap();
          ('█', Color::Rgb(pixel[2], pixel[1], pixel[0]))
        },
        ImageConvertType::GrayScale => {
          let intensity = frame.at_2d::<u8>(y, x).unwrap();
          let char_index = (*intensity as f32 * (ASCII_CHARS.len() - 1) as f32 / 255.0).round() as usize;

          (ASCII_CHARS[char_index], Color::Rgb(255, 255, 255))
        },
        ImageConvertType::Threshold => {
          let intensity = frame.at_2d::<u8>(y, x).unwrap();
          (if *intensity > 150 { '█' } else { ' ' }, Color::Rgb(255, 255, 255))
        }
      };

      spans.push(
        Span::from(ascii_char.to_string())
        .style(Style::default().fg(color))
      );
    }

    lines.push(Line::from(spans));
  }

  Text::from(lines)
}


pub struct FrameHandler;

impl FrameHandler {

  /// Spawns a new Tokio task.
  /// 
  /// This task opens a device camera, captures a frame, and resizes the image.
  /// If frame is a GrayScale or Threshold converts into approriate format
  pub fn try_new(
    config: Arc<RwLock<FrameHandlerConfig>>,
    tx: tokio::sync::mpsc::UnboundedSender<AppEvent>
  ) -> opencv::Result<Self> {
    let mut cam = VideoCapture::new(4, videoio::CAP_ANY)?;
    let mut frame = opencv::core::Mat::default();

    let _handle = tokio::spawn(async move {
      // Camera frame delay
      let mut interval = tokio::time::interval(Duration::from_millis(50));

      loop {
        cam.read(&mut frame).unwrap();

        let mut small_frame = opencv::core::Mat::default();

        let config = config.read().await;

        let cam_size = opencv::core::Size {
          width: (config.terminal_size.0 / config.cam_window_scale.clone() as u16) as i32,
          height: (config.terminal_size.1 / config.cam_window_scale.clone() as u16) as i32
        };

        opencv::imgproc::resize(
          &frame,
          &mut small_frame,
          cam_size,
          0.0,
          0.0,
          opencv::imgproc::INTER_LINEAR
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

        let ascii_frame = convert_frame_into_ascii(
          res_frame,
          config.image_convert_type.clone()
        );

        if tx.send(AppEvent::AsciiFrame(ascii_frame)).is_err() { break; }
        interval.tick().await;
      }
    });

    Ok(Self {})
  }
}

#[allow(unused)]
pub struct EventHandler(pub tokio::task::JoinHandle<()>);

impl EventHandler {
  /// Spawns a new Tokio task.
  ///
  /// This task waits on the Crossbeam event occur.
  /// The event consists of either a key event or a resize event.
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
