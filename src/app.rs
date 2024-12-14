use std::error;

use ratatui::{layout::{Alignment, Constraint, Flex, Layout}, style::Style, widgets::{Block, BorderType, Clear, Paragraph}, DefaultTerminal, Frame};

use crate::event_handler::{EventHandler, KeyAction};

pub struct App<'a> {
  // Base terminal
  terminal: &'a mut DefaultTerminal,

  // Evetn handler
  event_handler: EventHandler,

  // App is running
  is_running: bool
}

impl<'a> App<'a> {
  pub fn new(terminal: &'a mut DefaultTerminal, event_handler: EventHandler) -> Self {
    Self {
      terminal,
      event_handler,
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
      self.terminal.draw(|frame| {
        let area = frame.area();

        let block = Block::bordered()
          .style(Style::default())
          .border_type(BorderType::Rounded);

        let cam_paragraph = Paragraph::new("Hello")
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

      if let Some(key_action) = self.event_handler.next().await {
        match key_action {
          KeyAction::Exit => self.is_running = false,
          _ => {}
        }
      }
    }

    Ok(())
  }

}
