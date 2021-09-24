use anyhow::{Context, Result};

mod beeminder;
mod bot;
mod settings;

use settings::Settings;

#[tokio::main]
async fn main() -> Result<()> {
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 2 {
    return Err(anyhow::anyhow!(
      "Exactly 1 arg required, the config filename"
    ));
  }

  let settings = Settings::from(&args[1])?;
  bot::login_and_sync(&settings)
    .await
    .context("Failed to login to Matrix")?;
  Ok(())
}
