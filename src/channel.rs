use crossterm::event::KeyEvent;

pub enum AppEvent {
  Frame(opencv::core::Mat),
  Event(KeyEvent)
}

pub struct Channel {
  rx: tokio::sync::mpsc::UnboundedReceiver<AppEvent>,
  tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
}

impl Channel {
  pub fn new() -> Self {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<AppEvent>();

    Self {
      tx,
      rx
    }
  }

  pub fn get_tx(&mut self) -> tokio::sync::mpsc::UnboundedSender<AppEvent> {
    self.tx.clone()
  }

  pub async fn next(&mut self) -> Option<AppEvent> {
    self.rx.recv().await
  }
}
