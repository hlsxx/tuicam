mod app;
mod channel;
mod handler;

use app::App;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut terminal = ratatui::init();

  let app_result = App::try_new(&mut terminal)?
    .run()
    .await;

  ratatui::restore();

  app_result
}
