#![allow(incomplete_features)]
#![feature(backtrace,capture_disjoint_fields)]

use crate::bot::BOT_CLIENT;
use anyhow::Result;
use config::CONFIG;
use message::init_nc;
use serenity::{
  client::{bridge::gateway::GatewayIntents, ClientBuilder},
  framework::standard::StandardFramework,
};

#[macro_use]
extern crate log;
#[macro_use]
extern crate educe;
#[macro_use]
extern crate automatic_config;
#[macro_use]
extern crate singleton;

mod bot;
mod commands;
mod config;
mod data;
mod event;
mod framework;
mod message;
mod net;

fn main() {
  std::env::set_var("RUST_BACKTRACE", "1");
  std::backtrace::Backtrace::force_capture();
  env_logger::builder()
    .write_style(env_logger::WriteStyle::Auto)
    .filter(None, log::LevelFilter::Error)
    .format_timestamp(None)
    .filter(Some("discord_message_source"), log::LevelFilter::Trace)
    .filter(Some("mesagisto_client"), log::LevelFilter::Trace)
    .filter(Some("serenity"), log::LevelFilter::Info)
    .init();
  tokio::runtime::Builder::new_multi_thread()
  // fixme: how many do we need
    .worker_threads(5)
    .enable_all()
    .build()
    .unwrap()
    .block_on(async {
      run().await.unwrap();
    });
}

async fn run() -> Result<(), anyhow::Error> {
  init_nc();
  let framework = StandardFramework::new()
    .configure(|c| c.prefix("/"))
    .help(&framework::HELP)
    .group(&framework::MESAGISTO_GROUP)
    .normal_message(message::normal_message);

  if !CONFIG.enabled {
    log::info!("Mesagisto-Bot is not enabled and is about to exit the program");
    return Ok(());
  }

  let http = net::build_http().await;
  let mut client = ClientBuilder::new_with_http(http)
    .event_handler(event::Handler)
    .framework(framework)
    .intents({
      let mut intents = GatewayIntents::all();
      //intents.remove(GatewayIntents::GUILD_PRESENCES);
      intents.remove(GatewayIntents::DIRECT_MESSAGE_TYPING);
      intents.remove(GatewayIntents::GUILD_MESSAGE_TYPING);
      intents
    })
    .await
    .expect("Err creating client");
  BOT_CLIENT.init(client.cache_and_http.clone());
  // a shutdown handle task
  let shard_manager = client.shard_manager.clone();
  tokio::spawn(async move {
    tokio::signal::ctrl_c()
      .await
      .expect("Could not register ctrl+c handler");
    info!("Mesagisto Bot is shutting down");
    shard_manager.lock().await.shutdown_all().await;
    info!("Saving configuration file");
    CONFIG.save();
  });

  // start to dispatch events
  if let Err(why) = client.start().await {
    error!("Client error: {:?}", why);
  }
  Ok(())
}
