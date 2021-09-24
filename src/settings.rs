use anyhow::{Context, Result};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Settings {
  pub beeminder_username: String,
  pub beeminder_goal: String,
  pub beeminder_auth_token: String,
  pub matrix_homeserver_url: String,
  pub matrix_username: String,
  pub matrix_password: String,
}

impl Settings {
  pub fn from(file: &str) -> Result<Self> {
    let mut s = config::Config::new();
    s.merge(config::File::with_name(file))?;
    let ret: Self = s.try_into().context("Could not parse config file")?;
    Ok(ret)
  }
}
