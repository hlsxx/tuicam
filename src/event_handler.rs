use ratatui::crossterm::event::{Event, EventStream, KeyCode};
use tokio::{select, sync::mpsc, task::JoinHandle};
use futures::{FutureExt, StreamExt};

pub enum KeyAction {
  Exit,
  None,
  CamFrame
}

pub struct EventHandler {
  _tx: mpsc::UnboundedSender<KeyAction>,
  rx: mpsc::UnboundedReceiver<KeyAction>,
  _handle: tokio::task::JoinHandle<()>,
}

impl EventHandler {
  pub fn new() -> Self {
    let (_tx, rx) = mpsc::unbounded_channel::<KeyAction>();

    let tx_clone = _tx.clone();
    let _handle  = tokio::spawn(async move {
      let mut reader = EventStream::new();

      loop {
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

  pub fn get_handle(&self) -> &JoinHandle<()> {
    &self._handle
  }

  pub async fn next(&mut self) -> Option<KeyAction> {
    self.rx.recv().await
  }
}
