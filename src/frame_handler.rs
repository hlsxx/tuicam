use std::time::Duration;

use crossterm::event::{EventStream, KeyEvent, Event};
use tokio::{select, sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}};
use futures::{FutureExt, StreamExt};

use opencv::{prelude::*, imgproc, videoio::{self, VideoCapture, VideoCaptureTrait}};

const ASCII_CHARS: &[u8] = b"@%#*+=-:. ";

enum FrameEvent {
  Frame(String),
  Event(KeyEvent)
}

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

pub struct FrameHandler {
  _tx: UnboundedSender<FrameEvent>,
  rx: UnboundedReceiver<FrameEvent>
}

impl FrameHandler {
  pub fn try_new() -> opencv::Result<Self> {
    let mut cam = VideoCapture::new(0, videoio::CAP_ANY)?;
    let mut frame = opencv::core::Mat::default();
    let mut gray_frame = opencv::core::Mat::default();
    let mut small_frame = opencv::core::Mat::default();

    let (_tx, rx) = unbounded_channel::<FrameEvent>();

    let tx_clone = _tx.clone();
    let _handle = tokio::spawn(async move {
      let mut reader = EventStream::new();
      let mut interval = tokio::time::interval(Duration::from_millis(300));

      loop {
        let crossterm_event = reader.next().fuse();
        let frame_tick = interval.tick();

        select! {
          _ = frame_tick => {
            cam.read(&mut frame).unwrap();

            imgproc::cvt_color(&frame, &mut gray_frame, imgproc::COLOR_BGR2GRAY, 0).unwrap();

            imgproc::resize(
              &gray_frame,
              &mut small_frame,
              opencv::core::Size { width: 80, height: 40 }, 0.0, 0.0, imgproc::INTER_LINEAR
            ).unwrap();

            let image_ascii = image_to_ascii(&small_frame);
            tx_clone.send(FrameEvent::Frame(image_ascii)).unwrap();
          },
          Some(Ok(event)) = crossterm_event => {
            match event {
              Event::Key(key_code) => tx_clone.send(FrameEvent::Event(key_code)).unwrap(),
              _ => {}
            }
          }
        }
      }
    });

    Ok(Self {
      _tx,
      rx
    })

  }

  /*
  * Read into the buffer
  */
  pub async fn read_frame(&mut self, buffer: &mut String) {
    if let Some(FrameEvent::Frame(frame_ascii)) = self.rx.recv().await {
      buffer.push_str(frame_ascii.as_str());
    }
  }

  /*
  * Returns an occured key event
  */
  pub async fn get_event(&mut self) -> Option<KeyEvent> {
    if let Some(FrameEvent::Event(key_event)) = self.rx.recv().await {
      return Some(key_event);
    }

    None
  }

}
