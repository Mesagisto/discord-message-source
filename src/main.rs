#![allow(incomplete_features)]
#![feature(backtrace,capture_disjoint_fields)]

use std::error::Error;

use crate::bot::BOT_CLIENT;
use arcstr::ArcStr;
use mesagisto_client::{OptionExt, cache::CACHE, cipher::CIPHER, db::DB, res::RES, server::SERVER};
use anyhow::Result;
use config::CONFIG;
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
  if !CONFIG.enable {
    log::warn!("Mesagisto-Bot is not enabled and is about to exit the program.");
    log::warn!("To enable it, please modify the configuration file.");
    log::warn!("Mesagisto-Bot未被启用，即将退出程序。");
    log::warn!("若要启用，请修改配置文件。");
    return Ok(());
  }
  log::info!("Mesagisto-Bot is starting up");
  log::info!("Mesagisto-Bot正在启动");
  CACHE.init();
  if CONFIG.cipher.enable {
    CIPHER.init(&CONFIG.cipher.key,&CONFIG.cipher.refuse_plain);
  } else {
    CIPHER.deinit();
  }
  DB.init(ArcStr::from("dc").some());
  RES.init().await;
  // RES.resolve_photo_url(|id_pair| { } todo
  SERVER.init(&CONFIG.nats.address).await;
  let framework = StandardFramework::new()
    .configure(|c| c.prefix("/"))
    .help(&framework::HELP)
    .group(&framework::MESAGISTO_GROUP)
    .normal_message(message::handler::message_hook);

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
  // the coroutine will be suspend here until it stops
  client.start().await.expect("Client error");
  Ok(())
}
