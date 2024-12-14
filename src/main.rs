mod app;
mod event_handler;

use std::error;
use app::App;
use event_handler::EventHandler;

use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
  let mut terminal = ratatui::init();
  let mut event_handler = EventHandler::new();

  terminal.clear()?;

  let app_result = App::new(&mut terminal, event_handler)
    .run()
    .await;

  ratatui::restore();

  app_result
}
