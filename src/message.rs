use std::sync::Arc;

use crate::bot::BOT_CLIENT;
use crate::{config::CONFIG, data::DATA};
use nats::{asynk::Connection, Headers};
use once_cell::sync::Lazy;
use serenity::{
  client::Context,
  framework::standard::macros::hook,
  model::{channel::Message, id::ChannelId},
};

static NC: Lazy<Connection> = Lazy::new(|| {
  smol::block_on(async {
    let opts = nats::asynk::Options::new();
    log::info!("Connecting to nats server");
    let nc = opts
      .with_name("telegram client")
      .connect(&CONFIG.forwarding.address)
      .await
      .expect("Failed to connect nats server");
    log::info!("Connected sucessfully");
    nc
  })
});
static CID: Lazy<String> = Lazy::new(|| NC.client_id().to_string());
static NATS_HEADER: Lazy<Headers> = Lazy::new(|| {
  use std::collections::HashMap;
  use std::collections::HashSet;
  smol::block_on(async {
    let mut inner = HashMap::default();
    let entry = inner
      .entry("cid".to_string())
      .or_insert_with(HashSet::default);
    entry.insert(CID.clone());
    Headers { inner }
  })
});

pub fn init_nc() {
  Lazy::force(&NC);
  Lazy::force(&NATS_HEADER);
}

#[hook]
pub async fn normal_message(_: &Context, msg: &Message) {
  let target = Arc::new(msg.channel_id.as_u64().to_string());
  if CONFIG.target_address_mapper.contains_key(&target) {
    let address = CONFIG.target_address_mapper.get(&target).unwrap().clone();
    let sender = &msg.author;
    let content = format!("{}: {}", sender.name, msg.content);
    NC.publish_with_reply_or_headers(address.as_str(), None, Some(&NATS_HEADER), content)
      .await
      .unwrap();
    try_create_endpoint(target, address).await;
  }
}
async fn try_create_endpoint(target: Arc<String>, address: Arc<String>) {
  log::info!("Trying to create sub for {}", *target);
  if !DATA.active_endpoint.contains_key(&*target) {
    DATA.active_endpoint.insert(target.clone(), true);
    log::info!("Creating sub for {}", target);
    let sub = NC.subscribe(address.as_str()).await.unwrap();

    tokio::spawn(async move {
      loop {
        let next = sub.next().await;
        if next.is_none() {
          continue;
        }
        let next = next.unwrap();

        let headers = next.headers;
        if headers.is_none() {
          continue;
        }
        let headers = headers.unwrap();

        let cid_set = headers.get("cid");
        if cid_set.is_none() {
          continue;
        }
        let cid_set = cid_set.unwrap();

        if cid_set.contains(&*CID) {
          continue;
        }
        let data = next.data;
        let channel = ChannelId(target.as_str().parse::<u64>().unwrap());
        if let Err(err) = channel
          .send_message(&*BOT_CLIENT.http, |m| {
            m.content(String::from_utf8_lossy(&data));
            m
          })
          .await
        {
          log::error!("Serenity error {}", &err);
        };
      }
    });
  }
}
