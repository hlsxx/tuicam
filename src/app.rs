use crossterm::event::KeyCode;
use ratatui::{layout::{Alignment, Constraint, Direction, Flex, Layout}, style::{Color, Style, Stylize}, text::{Line, Span, Text}, widgets::{Block, BorderType, Clear, Paragraph}, DefaultTerminal};

use crate::channel::Channel;
use crate::channel::AppEvent;

pub const SCALE_FACTOR: u16 = 2;

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
          AppEvent::Frame(ascii_cam) => self.frame_buffer = ascii_cam,
          AppEvent::Event(key_event) => {
            match key_event.code {
              KeyCode::Esc => break,
              _ => {}
            }
          },
        }
      }

      self.terminal.draw(|frame| {
        let area = frame.area();

        let chunks = Layout::new(Direction::Vertical, [
          Constraint::Percentage(100),
          Constraint::Length(2)
        ]).split(area);

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
}
