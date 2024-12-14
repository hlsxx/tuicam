use crossterm::event::{EventStream, KeyCode};
use crossterm::event::Event::Key;
use tokio::sync::mpsc;
use futures::{select, FutureExt, StreamExt};

pub enum KeyAction {
  Exit,
  Another
}

pub struct EventHandler {
  _tx: mpsc::UnboundedSender<KeyAction>,
  rx: mpsc::UnboundedReceiver<KeyAction>
}

impl EventHandler {
  pub fn new() -> Self {
    let (_tx, rx) = mpsc::unbounded_channel::<KeyAction>();

    let tx_clone = _tx.clone();
    let _handle  = tokio::spawn(async move {
      let mut event = EventStream::new();

      loop {
        let crossterm_event = event.next().fuse().await;

        if let Some(Ok(event)) = crossterm_event {
          match event {
            Key(key_event) => {
              if key_event.code == KeyCode::Esc {
                let _ = tx_clone.send(KeyAction::Exit);
              }
            },
            _ => {}
          }
        }
      }
    });

    Self {
      _tx,
      rx
    }
  }

  pub async fn next(&mut self) -> Option<KeyAction> {
    self.rx.recv().await
  }
}
