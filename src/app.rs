use std::sync::{Arc, RwLock};

use crossterm::event::KeyCode;
use opencv::prelude::*;
use crate::channel::AppEvent;

use ratatui::{
  layout::{Alignment, Constraint, Direction, Flex, Layout},
  style::{Color, Style, Stylize},
  text::{Line, Span, Text},
  widgets::{Block, BorderType, Clear, Paragraph},
  DefaultTerminal
};


use crate::{
  channel::Channel,
  handler::{EventHandler, FrameHandler, FrameHandlerConfig, ImageConvertType}
};

/// Camera TUI frame scale (terminal.width() / SCALE_FACTOR)
/// Camera TUI frame scale (terminal.height() / SCALE_FACTOR)
pub const SCALE_FACTOR: u16 = 2;

/// Camera TUI frame border color
const PRIMARY_COLOR: Color = Color::Rgb(168, 50, 62);

const ASCII_CHARS: &[char] = &['█', '▓', '▒', '░', ' '];

pub struct App<'a> {
  // Base terminal
  terminal: &'a mut DefaultTerminal,

  // App channel
  channel: Channel,

  // Frame buffer (video buffer)
  frame_buffer: String,

  // Frame handler config (for a switchable image proccessing modes)
  frame_handler_config: Arc<RwLock<FrameHandlerConfig>>
}


impl<'a> App<'a> {

  /// Try to create new app
  ///
  /// App includes run method (render TUI)
  /// Try to creates a frame handler and event handler
  pub fn try_new(
    terminal: &'a mut DefaultTerminal
  ) -> Result<Self, Box<dyn std::error::Error>> {
    let mut channel = Channel::new();
    let frame_handler_config = Arc::new(RwLock::new(FrameHandlerConfig::default()));

    let _frame_handler = FrameHandler::try_new(frame_handler_config.clone(), channel.get_tx())?;
    let _event_handler = EventHandler::new(channel.get_tx());

    Ok(Self {
      terminal,
      channel,
      frame_buffer: String::new(),
      frame_handler_config
    })
  }

  /// Runs TUI application
  /// 
  /// Handles user events
  ///
  /// Renders widgets into frames
  pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    self.terminal.clear()?;

    loop {
      let terminal_size = self.terminal.size()?;

      if let Some(app_event) = self.channel.next().await {
        match app_event {
          AppEvent::Frame(frame) => {
            let image_size = opencv::core::Size {
              width: (terminal_size.width / SCALE_FACTOR) as i32,
              height: (terminal_size.height / SCALE_FACTOR) as i32
            };

            self.frame_buffer = self.convert_frame_into_ascii(image_size, frame);
          },
          AppEvent::Event(key_event) => {
            match key_event.code {
              KeyCode::Char(' ') => self.switch_mode(),
              KeyCode::Esc => break,
              _ => {}
            }
          },
        }
      }

      self.terminal.draw(|frame| {
        let area = frame.area();

        let chunks = Layout::new(Direction::Vertical, [
          Constraint::Percentage(100), Constraint::Length(2) ]).split(area);

        let top_chunk = chunks[0];
        let bottom_chunk = chunks[1];

        let block = Block::bordered()
          .border_style(Style::default().fg(PRIMARY_COLOR))
          .title_bottom(Line::from(" tui-cam-rs "))
          .title_style(Style::default())
          .title_alignment(Alignment::Center)
          .border_type(BorderType::Rounded);

        let cam_paragraph = Paragraph::new(self.frame_buffer.clone())
          .block(block)
          .alignment(Alignment::Center)
          .centered();

        let horizontal = Layout::horizontal([Constraint::Length(terminal_size.width / SCALE_FACTOR)])
          .flex(Flex::Center);

        let vertical = Layout::vertical([Constraint::Length(terminal_size.height / SCALE_FACTOR)])
          .flex(Flex::Center);

        let [top_chunk] = vertical.areas(top_chunk);
        let [top_chunk] = horizontal.areas(top_chunk);

        let tools_text = Text::from(vec![
          Line::from(vec![
            Span::from("ESC").bold(),
            Span::from(" exit | "),
            Span::from("[ __ ]").bold(),
            Span::from(" switch mode")
          ]).style(Style::default().fg(PRIMARY_COLOR))
        ]);

        let tools_paragraph = Paragraph::new(tools_text)
          .alignment(Alignment::Center)
          .centered();

        frame.render_widget(Clear, top_chunk);
        frame.render_widget(cam_paragraph, top_chunk);
        frame.render_widget(tools_paragraph, bottom_chunk);
      })?;
    }

    Ok(())
  }

  /// Converts a camera frame into ASCII
  ///
  /// Resize the frame to a smaller size
  ///
  /// Inserts an ASCII_CHAR based on the intensity
  pub fn convert_frame_into_ascii(
    &self,
    area_size: opencv::core::Size,
    frame: opencv::core::Mat
  ) -> String {
    let mut small_frame = opencv::core::Mat::default();

    opencv::imgproc::resize(
      &frame,
      &mut small_frame,
      area_size, 0.0, 0.0, opencv::imgproc::INTER_LINEAR
    ).unwrap();

    let mut ascii_image = String::new();

    let image_convert_type_guard = self.frame_handler_config.read().unwrap();

    for y in 0..small_frame.rows() {
      for x in 0..small_frame.cols() {
        let intensity = small_frame.at_2d::<u8>(y, x).unwrap();
        let ascii_char = if image_convert_type_guard.image_convert_type == ImageConvertType::Threshold {
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

  /// Switches a camera mode
  ///
  /// Startup mode: Image -> GrayScale -> ASCII
  /// Switch: Image -> GrayScale -> Threshold ->  ASCII
  pub fn switch_mode(&mut self) {
    let image_convert_type_guard = self.frame_handler_config.read().unwrap();

    let new_image_convert_type = if image_convert_type_guard.image_convert_type == ImageConvertType::GrayScale {
      ImageConvertType::Threshold
    } else {
      ImageConvertType::GrayScale
    };

    drop(image_convert_type_guard);

    self.frame_handler_config.write().unwrap().image_convert_type = new_image_convert_type;
  }
}
