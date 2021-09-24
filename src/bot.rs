use crate::beeminder::{get_goal, post_datapoint};
use crate::Settings;
use anyhow::{Context, Result};
use tokio::time::{sleep, Duration};

use matrix_sdk::{async_trait, room::Room, Client, ClientConfig, EventHandler, SyncSettings};

use ruma::events::{
  room::message::{MessageEventContent, MessageType, TextMessageEventContent},
  AnyMessageEventContent, SyncMessageEvent,
};
use url::Url;

const BOT_NAME: &'static str = "Beeminder";

fn parse_value(body: &str) -> Result<f64> {
  let val = body
    .trim()
    .parse()
    .context("Failed to parse request body into a value")?;
  Ok(val)
}

struct Bot {
  settings: Settings,
}

impl Bot {
  pub fn new(settings: Settings) -> Self {
    Self { settings }
  }
}

#[async_trait]
impl EventHandler for Bot {
  async fn on_room_message(&self, room: Room, event: &SyncMessageEvent<MessageEventContent>) {
    if let Room::Joined(room) = room {
      let msg_body = if let SyncMessageEvent {
        content:
          MessageEventContent {
            msgtype: MessageType::Text(TextMessageEventContent { body: msg_body, .. }),
            ..
          },
        ..
      } = event
      {
        msg_body
      } else {
        return;
      };

      // Try to read value from message
      let new_value = parse_value(msg_body);
      if let Err(_e) = new_value {
        return;
      }
      let new_value = new_value.unwrap();

      // Try to post the data to Beeminder
      let post_result = post_datapoint(&self.settings, new_value).await;
      if let Err(e) = post_result {
        if room
          .send(
            AnyMessageEventContent::RoomMessage(MessageEventContent::text_plain(format!(
              "Failed to update Beeminder: {}",
              e
            ))),
            None,
          )
          .await
          .is_err()
        {
          println!("Error responding to message.");
        }
        return;
      }

      // Try to send ACK to room
      if room
        .send(
          AnyMessageEventContent::RoomMessage(MessageEventContent::text_plain(
            "Uploaded new datapoint to Beeminder. Waiting to get new goal stats...",
          )),
          None,
        )
        .await
        .is_err()
      {
        println!("Error responding to message.");
        return;
      }

      // Wait 30 seconds
      sleep(Duration::from_millis(30 * 1000)).await;

      // Try to get information about the goal
      let get_result = get_goal(&self.settings).await;
      if let Err(e) = get_result {
        if room
          .send(
            AnyMessageEventContent::RoomMessage(MessageEventContent::text_plain(format!(
              "Failed to get goal details: {}",
              e
            ))),
            None,
          )
          .await
          .is_err()
        {
          println!("Error responding to message.");
        }
        return;
      }

      // Try to send the client information about the goal
      let get_result = get_result.unwrap();
      if room
        .send(
          AnyMessageEventContent::RoomMessage(MessageEventContent::text_html(
            format!(
              "You have {} days of buffer. See the graph at {}",
              &get_result.safe_buf, &get_result.graph_url
            ),
            format!(
              "You have {} days of buffer. See <a href=\"{}\">the graph</a> for more.",
              &get_result.safe_buf, &get_result.graph_url
            ),
          )),
          None,
        )
        .await
        .is_err()
      {
        println!("Error responding to message.");
      }
      return;
    }
  }
}

pub async fn login_and_sync(settings: &Settings) -> Result<()> {
  let client_config = ClientConfig::new();

  let homeserver_url =
    Url::parse(&settings.matrix_homeserver_url).context("Couldn't parse homeserver URL")?;
  let client = Client::new_with_config(homeserver_url, client_config).unwrap();

  client
    .login(
      &settings.matrix_username,
      &settings.matrix_password,
      None,
      Some(BOT_NAME),
    )
    .await
    .context("Failed to perform login")?;

  // An initial sync to set up state and so our bot doesn't respond to old
  // messages. If the `StateStore` finds saved state in the location given the
  // initial sync will be skipped in favor of loading state from the store
  client
    .sync_once(SyncSettings::default())
    .await
    .context("Failed initial Matrix sync")?;
  // add our CommandBot to be notified of incoming messages, we do this after the
  // initial sync to avoid responding to messages before the bot was running.
  client
    .set_event_handler(Box::new(Bot::new(settings.clone())))
    .await;
  // since we called `sync_once` before we entered our sync loop we must pass
  // that sync token to `sync`
  let settings = SyncSettings::default().token(
    client
      .sync_token()
      .await
      .context("Failed to pass sync token")?,
  );
  // this keeps state from the server streaming in to CommandBot via the
  // EventHandler trait
  client.sync(settings).await;

  Ok(())
}
