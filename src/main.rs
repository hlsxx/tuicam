mod app;
mod channel;
mod handler;

use app::App;

use channel::Channel;
use tokio;
use handler::{FrameHandler, EventHandler};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut terminal = ratatui::init();

  let mut channel = Channel::new();

  let _frame_handler = FrameHandler::try_new(channel.get_tx())?;
  let _event_handler = EventHandler::new(channel.get_tx());

  terminal.clear()?;

  let app_result = App::new(&mut terminal, channel)
    .run()
    .await;

  ratatui::restore();

  app_result
}
