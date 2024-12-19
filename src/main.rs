mod app;
mod frame_handler;

use std::error;
use app::App;

use tokio;
use frame_handler::FrameHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
  let mut terminal = ratatui::init();
  let frame_handler = FrameHandler::try_new()?;

  terminal.clear()?;

  let app_result = App::new(&mut terminal, frame_handler)
    .run()
    .await;

  ratatui::restore();

  app_result
}
