use std::{thread, time::Duration};

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use opencv::{prelude::*, imgproc, videoio::{self, VideoCapture, VideoCaptureTrait}};

const ASCII_CHARS: &[u8] = b"@%#*+=-:. ";

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

pub struct VideoCaptureHandler {
  _tx: UnboundedSender<String>,
  rx: UnboundedReceiver<String>
}

impl VideoCaptureHandler {
  pub fn try_new() -> opencv::Result<Self> {
    let mut cam = VideoCapture::new(0, videoio::CAP_ANY)?;
    let mut frame = opencv::core::Mat::default();
    let mut gray_frame = opencv::core::Mat::default();
    let mut small_frame = opencv::core::Mat::default();

    let (_tx, rx) = unbounded_channel::<String>();

    let tx_clone = _tx.clone();
    let _handle = tokio::spawn(async move {
      loop {
        cam.read(&mut frame).unwrap();

        imgproc::cvt_color(&frame, &mut gray_frame, imgproc::COLOR_BGR2GRAY, 0).unwrap();

        imgproc::resize(
          &gray_frame,
          &mut small_frame,
          opencv::core::Size { width: 80, height: 40 }, 0.0, 0.0, imgproc::INTER_LINEAR
        ).unwrap();

        let image_ascii = image_to_ascii(&small_frame);
        tx_clone.send(image_ascii).unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
      }
    });

    Ok(Self {
      _tx,
      rx
    })

  }
  pub async fn next(&mut self) -> Option<String> {
    self.rx.recv().await
  }
}
