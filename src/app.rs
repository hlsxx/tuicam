use std::{error, time::Duration};
use std::io::{self, Write};

use ratatui::{layout::{Alignment, Constraint, Flex, Layout}, style::Style, widgets::{Block, BorderType, Clear, Paragraph}, DefaultTerminal, Frame};

use opencv::{
  core, highgui, imgproc::{self, THRESH_BINARY}, prelude::*, videoio::{self, VideoCapture}
};
use tokio::try_join;

use crate::event_handler::{EventHandler, KeyAction};
use crate::video_capture_handler::VideoCaptureHandler;

fn clear_console() {
  print!("\x1b[2J\x1b[H");
  std::io::stdout().flush().unwrap();
}

fn image_to_ascii(frame: &Mat) -> String {
  const ASCII_CHARS: &[u8] = b"@%#*+=-:. ";
  let mut res = String::new();

  for y in 0..frame.rows() {
    for x in 0..frame.cols() {
      let intensity = frame.at_2d::<u8>(y, x).unwrap();

      let char_index = (*intensity as f32 * (ASCII_CHARS.len() - 1) as f32 / 255.0).round() as usize;
      let ascii_char = ASCII_CHARS[char_index] as char;

      res.push(ascii_char);
    }

    res.push_str("\n");
  }

  res
}

pub struct App<'a> {
  // Base terminal
  terminal: &'a mut DefaultTerminal,

  // Event handler
  event_handler: EventHandler,

  // Video capture handler
  video_capture_handler: VideoCaptureHandler,

  // App is running
  is_running: bool
}


impl<'a> App<'a> {
  pub fn new(
    terminal: &'a mut DefaultTerminal,
    event_handler: EventHandler,
    video_capture_handler: VideoCaptureHandler
  ) -> Self {
    Self {
      terminal,
      event_handler,
      video_capture_handler,
      is_running: true
    }
  }

  /// Runs TUI application
  /// 
  /// Handles user events
  ///
  /// Renders widgets into a frames
  pub async fn run(&mut self) -> Result<(), Box<dyn error::Error>> {
    while self.is_running {
      let mut frame_buffer = String::new();

      if let Some(value) = self.video_capture_handler.next().await {
        frame_buffer = value;
      }

      self.terminal.draw(|frame| {
        let area = frame.area();

        let block = Block::bordered()
          .style(Style::default())
          .border_type(BorderType::Rounded);

        let cam_paragraph = Paragraph::new(frame_buffer)
          .block(block)
          .alignment(Alignment::Center)
          .centered();

        let vertical = Layout::vertical([Constraint::Percentage(50)])
          .flex(Flex::Center);

        let horizontal = Layout::horizontal([Constraint::Percentage(50)])
          .flex(Flex::Center);

        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);

        frame.render_widget(Clear, area);
        frame.render_widget(cam_paragraph, area);
      })?;

      // if let Some(key_action) = self.event_handler.next().await {
      //   match key_action {
      //     KeyAction::Exit => self.is_running = false,
      //     _ => {}
      //   }
      // }
    }

    Ok(())
  }
}
