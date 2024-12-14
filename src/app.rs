use std::error;

use ratatui::{layout::{Constraint, Layout}, widgets::Paragraph, DefaultTerminal};

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

  pub async fn run(&mut self) -> Result<(), Box<dyn error::Error>> {
    while self.is_running {
      if let Some(key_action) = self.event_handler.next().await {
        match key_action {
          KeyAction::Exit => self.is_running = false,
          _ => {}
          }
      }

      self.terminal.draw(|frame| {
        let main_layout = Layout::default()
          .constraints(vec![
            Constraint::Percentage(50),
            Constraint::Percentage(50)
          ])
          .split(frame.area());

        let p = Paragraph::new("Hello");

        frame.render_widget(&p, main_layout[0]);
        frame.render_widget(p, main_layout[1]);
      })?;
    }

    Ok(())
  }
}
