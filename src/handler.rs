use std::{sync::Arc, time::Duration};

use crossterm::event::{Event, EventStream};
use futures::{FutureExt, StreamExt};
use ratatui::{
  layout::Size,
  style::{Color, Style},
  text::{Line, Span, Text},
};

use tokio::sync::RwLock;

use opencv::{
  core::VecN, imgproc, prelude::*, videoio::{self, VideoCapture, VideoCaptureTrait}
};

#[cfg(feature = "opencv_newer")]
use opencv::core::AlgorithmHint;

use crate::app::ASCII_CHARS;
use crate::channel::AppEvent;

type TerminalSize = (u16, u16);

#[derive(Eq, PartialEq, Clone)]
#[allow(unused)]
pub enum ImageConvertType {
  ColorfulHalfBlock,
  Colorful,
  GrayScale,
  GrayScaleThreshold,
  Threshold,
}

/// Camera window frame scale
#[derive(Clone)]
pub enum CamWindowScale {
  Full = 1,
  Small = 2,
}

/// Camera contains all available device cameras
pub struct Camera {
  pub(crate) active_index: Option<i32>,
  pub(crate) ids: Vec<i32>,
}

impl Camera {
  pub fn default() -> Self {
    let ids = (0..=10)
      .filter_map(|id| {
        if let Ok(cam) = VideoCapture::new(id, videoio::CAP_ANY) {
          if cam.is_opened().unwrap_or(false) {
            return Some(id);
          } else {
            return None;
          }
        }

        None
      })
      .collect::<Vec<i32>>();

    Self {
      active_index: Some(0),
      ids
    }
  }

  /// Switches current camera
  pub fn switch(&mut self) {
    if let Some(active_index) = self.active_index.as_mut() {
      if self.ids.len() > (*active_index + 1) as usize {
        *active_index += 1;
      } else {
        *active_index = 0;
      }
    }
  }

  pub fn get_cam_id(&self) -> Option<&i32> {
    if let Some(active_index) = self.active_index.as_ref() {
      self.ids.get(*active_index as usize)
    } else {
      None
    }
  }
}

/// Frame handler config
pub struct FrameHandlerConfig {
  /// Image convert type (camera mode)
  pub image_convert_type: ImageConvertType,

  /// Terminal size (width, height)
  pub terminal_size: TerminalSize,

  /// Camera window scale
  pub cam_window_scale: CamWindowScale,

  /// Active camera id
  pub camera: Camera,
}

impl FrameHandlerConfig {
  pub fn new(terminal_size: Size) -> Self {
    Self {
      image_convert_type: ImageConvertType::ColorfulHalfBlock,
      terminal_size: (terminal_size.width, terminal_size.height),
      cam_window_scale: CamWindowScale::Small,
      camera: Camera::default(),
    }
  }
}

/// Converts a frame into a grayscale.
fn convert_into_grayscale(frame: &opencv::core::Mat, res_frame: &mut opencv::core::Mat) {
  #[cfg(feature = "opencv_newer")]
  {
    imgproc::cvt_color(
      frame,
      res_frame,
      imgproc::COLOR_BGR2GRAY,
      0,
      AlgorithmHint::ALGO_HINT_DEFAULT,
    ).unwrap();
  }

  #[cfg(not(feature = "opencv_newer"))]
  {
    imgproc::cvt_color(frame, res_frame, imgproc::COLOR_BGR2GRAY, 0).unwrap();
  }
}

/// Computes the distance between two colors
fn color_dist(lhs: &[u8; 3], rhs: &[u8; 3]) -> u32 {
  let x = lhs[0].abs_diff(rhs[0]) as u32;
  let y = lhs[1].abs_diff(rhs[1]) as u32;
  let z = lhs[2].abs_diff(rhs[2]) as u32;
  x * x + y * y + z * z
}

/// Computes the distance between two colors
fn color_average<const N: usize>(colors: [[u8; 3]; N]) -> [u8; 3] {
  let x = colors.iter().map(|color| color[0] as u32).sum::<u32>() / (N as u32);
  let y = colors.iter().map(|color| color[1] as u32).sum::<u32>() / (N as u32);
  let z = colors.iter().map(|color| color[2] as u32).sum::<u32>() / (N as u32);
  [x as u8, y as u8, z as u8]
}

/// Converts a camera frame into ASCII frame.
///
/// This method resizes the frame to a smaller size and then converts each pixel
/// into an ASCII character based on its intensity. The intensity is calculated
/// from the pixel's RGB values (Colorful), and the corresponding ASCII character is inserted
/// based on that intensity.
pub fn convert_frame_into_ascii(
  frame: opencv::core::Mat,
  image_convert_type: ImageConvertType,
) -> Text<'static> {
  let mut lines = Vec::new();

  let (width, height) = match image_convert_type {
    ImageConvertType::ColorfulHalfBlock => (frame.cols() / 2, frame.rows() / 2),
    _ => (frame.cols(), frame.rows()),
  };

  for y in 0..height {
    let mut spans = Vec::new();
    for x in 0..width {
      let (ascii_char, fg_color, bg_color) = match image_convert_type {
        ImageConvertType::ColorfulHalfBlock => {
          // Sub-character colors are arranged as shown below.
          //
          //  0 | 1
          //  --+--
          //  2 | 3
          let subpixels: [_; 4] = std::array::from_fn(|i| {
            let i = i as i32;

            let pixel = frame
              .at_2d::<opencv::core::Vec3b>(y * 2 + i / 2, x * 2 + i % 2)
              .cloned()
              .unwrap_or(opencv::core::Vec3b::from([255, 255, 255]));

            [pixel[0], pixel[1], pixel[2]]
          });

          // Find the two nearest subpixels.  These two will define
          // the foreground color.
          let (fg1, fg2, other1, other2) = [
            (0, 1, 2, 3),
            (0, 2, 1, 3),
            (0, 3, 1, 2),
            (1, 2, 0, 3),
            (1, 3, 0, 2),
            (2, 3, 0, 1),
          ]
          .into_iter()
          .min_by_key(|(i, j, _, _)| color_dist(&subpixels[*i], &subpixels[*j]))
          .unwrap();

          let fg_color = color_average([subpixels[fg1], subpixels[fg2]]);

          // Of the two remaining colors, is one of them closer to the
          // foreground color than they are to each other?
          let dist_remaining = color_dist(&subpixels[other1], &subpixels[other2]);
          let dist_1 = color_dist(&fg_color, &subpixels[other1]);
          let dist_2 = color_dist(&fg_color, &subpixels[other2]);

          let (fg_char, fg_color, bg_color) = if dist_remaining < dist_1 && dist_remaining < dist_2
          {
            // The two remaining colors are closer to each other
            // than they are to the foreground color.  They will
            // both share the same background color.
            let bg_color = color_average([subpixels[other1], subpixels[other2]]);
            let fg_char = match (fg1, fg2) {
              (0, 1) => '▀',
              (0, 2) => '▌',
              (0, 3) => '▚',
              (1, 2) => '▞',
              (1, 3) => '▐',
              (2, 3) => '▄',
              _ => unreachable!(),
            };
            (fg_char, fg_color, bg_color)
          } else if dist_1 < dist_2 {
            // The point at `other1` is close to the fg color.
            // Therefore, average those three into the foreground, and
            // use the background for the last color.
            let fg_color = color_average([subpixels[fg1], subpixels[fg2], subpixels[other1]]);
            let bg_color = subpixels[other2];
            let fg_char = match other2 {
              0 => '▟',
              1 => '▙',
              2 => '▜',
              3 => '▛',
              _ => unreachable!(),
            };

            (fg_char, fg_color, bg_color)
          } else {
            // The point at `other2` is close to the fg color.
            // Therefore, average those three into the foreground, and
            // use the background for the last color.
            let fg_color = color_average([subpixels[fg1], subpixels[fg2], subpixels[other2]]);
            let bg_color = subpixels[other1];
            let fg_char = match other1 {
              0 => '▟',
              1 => '▙',
              2 => '▜',
              3 => '▛',
              _ => unreachable!(),
            };
            (fg_char, fg_color, bg_color)
          };

          (
            fg_char,
            Color::Rgb(fg_color[2], fg_color[1], fg_color[0]),
            Color::Rgb(bg_color[2], bg_color[1], bg_color[0]),
          )
        }
        ImageConvertType::Colorful => {
          let pixel = frame.at_2d::<opencv::core::Vec3b>(y, x).unwrap();
          ('█', Color::Rgb(pixel[2], pixel[1], pixel[0]), Color::Reset)
        }
        ImageConvertType::GrayScale => {
          let intensity = frame.at_2d::<u8>(y, x).unwrap();

          (
            '█',
            Color::Rgb(*intensity, *intensity, *intensity),
            Color::Reset,
          )
        }
        ImageConvertType::GrayScaleThreshold => {
          let intensity = frame.at_2d::<u8>(y, x).unwrap();
          let char_index =
            (*intensity as f32 * (ASCII_CHARS.len() - 1) as f32 / 255.0).round() as usize;

          (
            ASCII_CHARS[char_index],
            Color::Rgb(255, 255, 255),
            Color::Reset,
          )
        }
        ImageConvertType::Threshold => {
          let intensity = frame.at_2d::<u8>(y, x).unwrap();
          (
            if *intensity > 150 { '█' } else { ' ' },
            Color::Rgb(255, 255, 255),
            Color::Reset,
          )
        }
      };

      let style = Style::default().fg(fg_color).bg(bg_color);
      spans.push(Span::from(ascii_char.to_string()).style(style));
    }

    lines.push(Line::from(spans));
  }

  Text::from(lines)
}

pub struct FrameHandler {
  config: Arc<RwLock<FrameHandlerConfig>>,
  tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
}

impl FrameHandler {
  pub async fn try_new(
    config: Arc<RwLock<FrameHandlerConfig>>,
    tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
  ) -> opencv::Result<Self> {
    Ok(Self {
      config,
      tx
    })
  }

  pub fn get_cam(
    &self,
    cam_id: i32,
    cam: &mut Option<VideoCapture>
  ) {
    *cam = Some(VideoCapture::new(
      cam_id,
      videoio::CAP_ANY,
    ).unwrap());
  }

  /// Spawns a new Tokio task.
  ///
  /// This task opens a device camera, captures a frame, and resizes the image.
  /// If frame is a GrayScale or Threshold converts into approriate format
  pub async fn run(self) -> opencv::Result<()> {
    let _handle = tokio::spawn(async move {
      let (mut cam, mut active_cam_id) = (None, -1);

      let mut frame = opencv::core::Mat::default();
      let mut interval = tokio::time::interval(Duration::from_millis(50));

      loop {
        let mut small_frame = opencv::core::Mat::default();

        let current_cam_id = self.config
          .read()
          .await
          .camera
          .get_cam_id()
          .unwrap()
          .clone();

        if current_cam_id != active_cam_id {
          self.get_cam(current_cam_id, &mut cam);
          active_cam_id = current_cam_id;
        }

        cam.as_mut().unwrap().read(&mut frame).unwrap();

        let cam_size = {
          let config = self.config.read().await;

          let cam_size = opencv::core::Size {
            width: (config.terminal_size.0 / config.cam_window_scale.clone() as u16) as i32,
            height: (config.terminal_size.1 / config.cam_window_scale.clone() as u16) as i32,
          };

          match config.image_convert_type {
            ImageConvertType::ColorfulHalfBlock => opencv::core::Size {
              width: cam_size.width * 2,
              height: cam_size.height * 2,
            },
            _ => cam_size,
          }
        };

        // Some virtual cams crash on the resize call.
        // If some error occurs just switch to an another cam.
        if let Err(_) = opencv::imgproc::resize(
          &frame,
          &mut small_frame,
          cam_size,
          0.0,
          0.0,
          opencv::imgproc::INTER_LINEAR,
        ) {
          self.config.write().await.camera.switch();
          continue;
        }

        let config = self.config.read().await;
        let res_frame = match config.image_convert_type {
          ImageConvertType::Colorful | ImageConvertType::ColorfulHalfBlock => small_frame.clone(),
          ImageConvertType::GrayScale | ImageConvertType::GrayScaleThreshold => {
            let mut gray_frame = opencv::core::Mat::default();
            convert_into_grayscale(&small_frame, &mut gray_frame);
            gray_frame
          }
          ImageConvertType::Threshold => {
            let mut gray_frame = opencv::core::Mat::default();
            let mut binary_frame = opencv::core::Mat::default();

            convert_into_grayscale(&small_frame, &mut gray_frame);

            imgproc::threshold(
              &gray_frame,
              &mut binary_frame,
              128.0,
              255.0,
              imgproc::THRESH_BINARY,
            )
            .unwrap();

            binary_frame
          }
        };

        let ascii_frame = convert_frame_into_ascii(res_frame, config.image_convert_type.clone());

        if self.tx.send(AppEvent::AsciiFrame(ascii_frame)).is_err() {
          break;
        }

        interval.tick().await;
      }
    });

    Ok(())
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
            Event::Resize(width, height) => {
              tx.send(AppEvent::TerminalResize((width, height))).unwrap()
            }
            _ => {}
          }
        }
      }
    });

    Self(handle)
  }
}
