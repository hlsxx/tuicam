use crossterm::event::KeyCode;
use ratatui::{layout::{Alignment, Constraint, Direction, Flex, Layout}, style::{Color, Style, Stylize}, text::{Line, Span, Text}, widgets::{Block, BorderType, Clear, Paragraph}, DefaultTerminal};

use opencv::prelude::*;

use crate::channel::Channel;
use crate::channel::AppEvent;

pub const SCALE_FACTOR: u16 = 2;
const ASCII_CHARS: &[u8] = b"@%#*+=-:. ";

pub struct App<'a> {
  // Base terminal
  terminal: &'a mut DefaultTerminal,

  // App channel
  channel: Channel,

  // Frame buffer (video buffer)
  frame_buffer: String
}


impl<'a> App<'a> {

  pub fn new(
    terminal: &'a mut DefaultTerminal,
    channel: Channel,
  ) -> Self {
    Self {
      terminal,
      channel,
      frame_buffer: String::new()
    }
  }

  /// Runs TUI application
  /// 
  /// Handles user events
  ///
  /// Renders widgets into frames
  pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let primary_color = Color::Rgb(168, 50, 62);

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
              KeyCode::Esc => {
                break;
              },
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
          .border_style(Style::default().fg(primary_color))
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
          ]).style(Style::default().fg(primary_color))
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

  pub fn convert_frame_into_ascii(&self, area_size: opencv::core::Size, frame: opencv::core::Mat) -> String {
    let mut small_frame = opencv::core::Mat::default();
    // let mut binary_frame = opencv::core::Mat::default();

    // opencv::imgproc::threshold(&frame, &mut binary_frame, 128.0, 255.0, THRESH_BINARY).unwrap();

    opencv::imgproc::resize(
      &frame,
      &mut small_frame,
      area_size, 0.0, 0.0, opencv::imgproc::INTER_LINEAR
    ).unwrap();

    let mut ascii_image = String::new();

    for y in 0..small_frame.rows() {
      for x in 0..small_frame.cols() {
        let intensity = small_frame.at_2d::<u8>(y, x).unwrap();
        let char_index = (*intensity as f32 * (ASCII_CHARS.len() - 1) as f32 / 255.0).round() as usize;
        let ascii_char = ASCII_CHARS[char_index] as char;

        ascii_image.push(ascii_char);
      }

      ascii_image.push_str("\n");
    }

    ascii_image
  }
}
