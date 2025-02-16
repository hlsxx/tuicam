mod app;
mod channel;
mod handler;

use app::App;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut terminal = ratatui::init();

  opencv::core::set_log_level(opencv::core::LogLevel::LOG_LEVEL_SILENT)?;

  let app_result = App::try_new(&mut terminal).await?.run().await;

  ratatui::restore();

  app_result
}
