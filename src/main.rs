mod app;
mod event_handler;
mod video_capture_handler;

use std::error;
use app::App;
use event_handler::EventHandler;

use tokio;
use video_capture_handler::VideoCaptureHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
  let mut terminal = ratatui::init();
  let event_handler = EventHandler::new();
  let video_capture_handler = VideoCaptureHandler::try_new()?;

  terminal.clear()?;

  let app_result = App::new(&mut terminal, event_handler, video_capture_handler)
    .run()
    .await;

  ratatui::restore();

  app_result
}
