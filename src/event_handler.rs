use std::time::Duration;

use ratatui::crossterm::event::{Event, EventStream, KeyCode};
use tokio::{select, sync::mpsc};
use futures::{FutureExt, StreamExt};

use std::{error, time::Duration};
use std::io::{self, Write};

use ratatui::{layout::{Alignment, Constraint, Flex, Layout}, style::Style, widgets::{Block, BorderType, Clear, Paragraph}, DefaultTerminal, Frame};

use opencv::{
  core, highgui, imgproc::{self, THRESH_BINARY}, prelude::*, videoio::{self, VideoCapture}
};

pub enum KeyAction {
  Exit,
  None,
  CamFrame
}

pub struct EventHandler {
  // Cam
  cam: Option<VideoCapture>,

  _tx: mpsc::UnboundedSender<KeyAction>,
  rx: mpsc::UnboundedReceiver<KeyAction>,
  _handle: tokio::task::JoinHandle<()>,
}

impl EventHandler {
  pub fn new() -> Self {
    let tick_rate = Duration::from_millis(300);
    let (_tx, rx) = mpsc::unbounded_channel::<KeyAction>();

    let tx_clone = _tx.clone();
    let _handle  = tokio::spawn(async move {
      let mut reader = EventStream::new();
      let mut interval = tokio::time::interval(tick_rate);

      loop {
        let tick_delay = interval.tick();
        let crossterm_event = reader.next().fuse();

        select! {
          Some(Ok(event)) = crossterm_event => {
            match event {
              Event::Key(key_event) => {
                if key_event.code == KeyCode::Esc {
                  let _ = tx_clone.send(KeyAction::Exit);
                }
              },
              _ => {}
            }
          },
          _ = tick_delay => {

          }
        }
      }
    });

    Self {
      _tx,
      rx,
      _handle
    }
  }

  pub async fn next(&mut self) -> Option<KeyAction> {
    self.rx.recv().await
  }
}
