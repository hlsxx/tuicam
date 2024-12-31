use crossterm::event::KeyEvent;

pub enum AppEvent {
  // OpenCV mat (camera video frame)
  Frame(opencv::core::Mat),

  // Crossterm event
  Event(KeyEvent)
}

pub struct Channel {
  // Receiver
  rx: tokio::sync::mpsc::UnboundedReceiver<AppEvent>,

  // Transceiver
  tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
}

impl Channel {

  /// Creates a unbounded channel
  pub fn new() -> Self {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<AppEvent>();

    Self {
      tx,
      rx
    }
  }

  /// Returns transceiver
  pub fn get_tx(&mut self) -> tokio::sync::mpsc::UnboundedSender<AppEvent> {
    self.tx.clone()
  }

  /// Returns AppEvent
  pub async fn next(&mut self) -> Option<AppEvent> {
    self.rx.recv().await
  }
}
