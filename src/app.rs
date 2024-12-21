use crossterm::event::KeyCode;
use ratatui::{layout::{Alignment, Constraint, Flex, Layout}, style::{Color, Style}, text::Line, widgets::{Block, BorderType, Clear, Paragraph}, DefaultTerminal};

use crate::frame_handler::FrameHandler;
pub const SCALE_FACTOR: u16 = 2;

pub struct App<'a> {
  // Base terminal
  terminal: &'a mut DefaultTerminal,

  // Video capture handler
  frame_handler: FrameHandler,
}


impl<'a> App<'a> {

  pub fn new(
    terminal: &'a mut DefaultTerminal,
    frame_handler: FrameHandler
  ) -> Self {
    Self {
      terminal,
      frame_handler,
    }
  }

  /// Runs TUI application
  /// 
  /// Handles user events
  ///
  /// Renders widgets into frames
  pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    loop {
      let terminal_size = self.terminal.size()?;
      let mut frame_buffer = String::new();

      if let Some(key_event) = self.frame_handler.get_event().await {
        match key_event.code {
          KeyCode::Esc => break,
          _ => {}
        }
      }

      self.frame_handler.read_frame(&mut frame_buffer).await;

      self.terminal.draw(|frame| {
        let area = frame.area();

        let block = Block::bordered()
          .border_style(Style::default().fg(Color::Rgb(168, 50, 62)))
          .title_bottom(Line::from(" tui-cam-rs "))
          .title_style(Style::default())
          .title_alignment(Alignment::Center)
          .border_type(BorderType::Rounded);

        let cam_paragraph = Paragraph::new(frame_buffer)
          .block(block)
          .alignment(Alignment::Center)
          .centered();

        let horizontal = Layout::horizontal([Constraint::Length(terminal_size.width / SCALE_FACTOR)])
          .flex(Flex::Center);

        let vertical = Layout::vertical([Constraint::Length(terminal_size.height / SCALE_FACTOR)])
          .flex(Flex::Center);

        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);

        frame.render_widget(Clear, area);
        frame.render_widget(cam_paragraph, area);
      })?;
    }

    Ok(())
  }
}
