use std::sync::Arc;
use crossterm::event::KeyModifiers;
use tokio::sync::RwLock;

use crate::{channel::AppEvent, handler::CamWindowScale};

use ratatui::{
  crossterm::event::KeyCode,
  layout::{Alignment, Constraint, Direction, Flex, Layout},
  style::{Color, Style, Stylize},
  text::{Line, Span, Text},
  widgets::{Block, BorderType, Clear, Paragraph},
  DefaultTerminal,
};

use crate::{
  channel::Channel,
  handler::{EventHandler, FrameHandler, FrameHandlerConfig, ImageConvertType},
};

/// Camera TUI frame border color
const PRIMARY_COLOR: Color = Color::Rgb(230, 143, 106);

pub const ASCII_CHARS: &[char] = &['█', '▓', '▒', '░', ' '];

pub struct App<'a> {
  // Base terminal
  terminal: &'a mut DefaultTerminal,

  // App channel
  channel: Channel,

  // Frame buffer (video buffer)
  frame_buffer: Text<'static>,

  // Frame handler config (for a switchable image proccessing modes)
  frame_handler_config: Arc<RwLock<FrameHandlerConfig>>,
}

impl<'a> App<'a> {
  /// Try to create new app
  ///
  /// App includes run method (render TUI)
  /// Try to creates a frame handler and event handler
  pub async fn try_new(
    terminal: &'a mut DefaultTerminal,
  ) -> Result<Self, Box<dyn std::error::Error>> {
    let mut channel = Channel::new();
    let terminal_size = terminal.size()?;

    let frame_handler_config = Arc::new(RwLock::new(FrameHandlerConfig::new(terminal_size)));

    let frame_handler =
      FrameHandler::try_new(frame_handler_config.clone(), channel.get_tx()).await?;

    frame_handler.run().await?;

    let _event_handler = EventHandler::new(channel.get_tx());

    Ok(Self {
      terminal,
      channel,
      frame_buffer: Text::default(),
      frame_handler_config,
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
          AppEvent::AsciiFrame(ascii_frame) => self.frame_buffer = ascii_frame,
          AppEvent::Event(key_event) => {
            if key_event.modifiers.contains(KeyModifiers::CONTROL) && key_event.code == KeyCode::Char(' ') {
              self.toggle_lock().await;
            }

            if !self.frame_handler_config.read().await.is_locked {
              match key_event.code {
                KeyCode::Char('m') => self.switch_mode().await,
                KeyCode::Char('f') => self.switch_cam_window_scale().await,
                KeyCode::Char('c') => self.switch_cam().await,
                KeyCode::Esc => break,
                _ => {}
              }
            }
          },
          AppEvent::TerminalResize((width, height)) => {
            self.frame_handler_config.write().await.terminal_size = (width, height);
          }
        }
      }

      let cam_window_scale = self
        .frame_handler_config
        .read()
        .await
        .cam_window_scale
        .clone() as u16;

      let is_locked = self.frame_handler_config.read().await.is_locked;

      self.terminal.draw(|frame| {
        let area = frame.area();

        let chunks = Layout::new(
          Direction::Vertical,
          [Constraint::Percentage(100), Constraint::Length(2)],
        )
        .split(area);

        let top_chunk = chunks[0];
        let bottom_chunk = chunks[1];

        let block = Block::bordered()
          .border_style(Style::default().fg(PRIMARY_COLOR))
          .title_style(Style::default())
          .title_alignment(Alignment::Center)
          .border_type(BorderType::Rounded);

        let cam_paragraph = Paragraph::new(self.frame_buffer.clone())
          .block(block)
          .alignment(Alignment::Center)
          .centered();

        let horizontal =
          Layout::horizontal([Constraint::Length(terminal_size.width / cam_window_scale)])
            .flex(Flex::Center);

        let vertical =
          Layout::vertical([Constraint::Length(terminal_size.height / cam_window_scale)])
            .flex(Flex::Center);

        let [top_chunk] = vertical.areas(top_chunk);
        let [top_chunk] = horizontal.areas(top_chunk);

        let tools_text = Text::from(vec![Line::from(vec![
          Span::from("ESC").bold(),
          Span::from(" exit | "),
          Span::from("m").bold(),
          Span::from(" switch mode | "),
          Span::from("c").bold(),
          Span::from(" switch camera | "),
          Span::from("f").bold(),
          Span::from(" toggle fullscreen | "),
          Span::from("ctrl-<space>").bold(),
          Span::from(" toggle lock"),
        ])
        .style(Style::default().fg(PRIMARY_COLOR))]);

        let tools_paragraph = Paragraph::new(tools_text)
          .alignment(Alignment::Center)
          .centered();

        frame.render_widget(Clear, top_chunk);
        frame.render_widget(cam_paragraph, top_chunk);

        if !is_locked {
          frame.render_widget(tools_paragraph, bottom_chunk);
        }
      })?;
    }

    Ok(())
  }

  /// Switches a camera mode.
  ///
  /// Startup mode: Image -> GrayScale -> ASCII
  /// Switch: Image -> GrayScale -> Threshold ->  ASCII
  pub async fn switch_mode(&mut self) {
    let new_image_convert_type = match self.frame_handler_config.read().await.image_convert_type {
      ImageConvertType::ColorfulHalfBlock => ImageConvertType::Colorful,
      ImageConvertType::Colorful => ImageConvertType::GrayScale,
      ImageConvertType::GrayScale => ImageConvertType::GrayScaleThreshold,
      ImageConvertType::GrayScaleThreshold => ImageConvertType::Threshold,
      ImageConvertType::Threshold => ImageConvertType::ColorfulHalfBlock,
    };

    self.frame_handler_config.write().await.image_convert_type = new_image_convert_type;
  }

  /// Toggles lock mode.
  ///
  /// Hide/Show the bottom inctruction widget.
  /// Disables keyboard events.
  pub async fn toggle_lock(&mut self) {
    let mut config = self.frame_handler_config.write().await;
    config.is_locked = !config.is_locked;
  }

  /// Switches a camera window scale.
  ///
  /// Startup mode: Small
  /// Switch: Full
  pub async fn switch_cam_window_scale(&mut self) {
    let cam_window_scale = match self.frame_handler_config.read().await.cam_window_scale {
      CamWindowScale::Small => CamWindowScale::Full,
      CamWindowScale::Full => CamWindowScale::Small,
    };

    self.frame_handler_config.write().await.cam_window_scale = cam_window_scale;
  }

  /// Switches a device camera
  pub async fn switch_cam(&mut self) {
    self.frame_handler_config.write().await.camera.switch();
  }
}
