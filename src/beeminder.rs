use crate::Settings;
use anyhow::{Context, Result};
use chrono::prelude::*;

#[derive(Debug, Default, serde::Serialize)]
struct DatapointRequest {
  value: f64,
  comment: Option<String>,
  timestamp: Option<i64>,
  daystamp: Option<String>,
  #[serde(rename = "requestid")]
  request_id: Option<String>,
  auth_token: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Datapoint {
  pub id: String,
  pub timestamp: i64,
  pub daystamp: String,
  pub value: f64,
  pub comment: Option<String>,
  pub updated_at: i64,
  #[serde(rename = "requestid")]
  pub request_id: Option<String>,
}

async fn post_beeminder(settings: &Settings, point: &DatapointRequest) -> Result<Datapoint> {
  let client = reqwest::Client::new();
  let res = client
    .post(format!(
      "https://www.beeminder.com/api/v1/users/{}/goals/{}/datapoints.json",
      settings.beeminder_username, settings.beeminder_goal
    ))
    .form(&point)
    .send()
    .await
    .context("Failed to POST to Beeminder")?;

  if !res.status().is_success() {
    return Err(anyhow::anyhow!(format!(
      "Failed to POST to Beeminder. Returned: {}",
      res.status()
    )));
  }

  let res = res.json().await.context("Failed to deserialize response")?;
  Ok(res)
}

fn get_daily_request_id() -> String {
  let local: DateTime<Local> = Local::now();
  local.format("BOT_%Y-%m-%d").to_string()
}

pub async fn post_datapoint(settings: &Settings, value: f64) -> Result<Datapoint> {
  let req = DatapointRequest {
    value: value,
    request_id: Some(get_daily_request_id()),
    auth_token: settings.beeminder_auth_token.clone(),
    ..Default::default()
  };

  post_beeminder(settings, &req).await
}

pub async fn get_goal(settings: &Settings) -> Result<Goal> {
  let req = GoalRequest {
    auth_token: settings.beeminder_auth_token.clone(),
  };

  let client = reqwest::Client::new();
  let res = client
    .get(format!(
      "https://www.beeminder.com/api/v1/users/{}/goals/{}.json",
      settings.beeminder_username, settings.beeminder_goal
    ))
    .query(&req)
    .send()
    .await
    .context("Failed to GET Beeminder")?;

  if !res.status().is_success() {
    return Err(anyhow::anyhow!(format!(
      "Failed to GET Beeminder. Returned: {}",
      res.status()
    )));
  }

  let res = res.json().await.context("Failed to deserialize response")?;
  Ok(res)
}

#[derive(Debug, Default, serde::Serialize)]
struct GoalRequest {
  auth_token: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Goal {
  pub slug: String,
  // pub updated_at: i64,
  // pub title: String,
  // #[serde(rename = "fineprint")]
  // pub fine_print: Option<String>,
  // #[serde(rename = "yaxis")]
  // pub y_axis: String,
  // #[serde(rename = "goaldate")]
  // pub goal_date: i64,
  // #[serde(rename = "goalval")]
  // pub goal_val: f64,
  // pub rate: f64,
  // #[serde(rename = "r_units")]
  // pub runits: String, // TODO: Enum
  // pub svg_url: String,
  pub graph_url: String,
  // pub thumb_url: String,
  // #[serde(rename = "autodata")]
  // pub auto_data: String,
  // pub goal_type: String, // TODO: Enum
  // #[serde(rename = "losedate")]
  // pub lose_date: i64,
  // pub queued: bool,
  // pub secret: bool,
  // #[serde(rename = "datapublic")]
  // pub data_public: bool,
  #[serde(rename = "safebuf")]
  pub safe_buf: i64,
  // TODO: A lot more fields
}
