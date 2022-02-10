#![allow(incomplete_features)]
#![feature(backtrace, capture_disjoint_fields)]

use crate::bot::{DcFile, BOT_CLIENT};
use anyhow::Result;
use config::CONFIG;
use mesagisto_client::MesagistoConfig;
use pretty_env_logger::env_logger::{self, TimestampPrecision};
use serenity::{
  client::{ClientBuilder, bridge::gateway::GatewayIntents}, framework::standard::StandardFramework,
};
use smol::future::FutureExt;

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
pub mod ext;
mod framework;
mod message;
mod net;

fn main() {
  std::backtrace::Backtrace::force_capture();
  env_logger::builder()
    .write_style(env_logger::WriteStyle::Auto)
    .filter(None, log::LevelFilter::Error)
    .format_timestamp(Some(TimestampPrecision::Seconds))
    .filter(Some("discord_message_source"), log::LevelFilter::Info)
    .filter(Some("mesagisto_client"), log::LevelFilter::Info)
    .filter(Some("serenity"), log::LevelFilter::Warn)
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

  MesagistoConfig::builder()
    .name("dc")
    .cipher_enable(CONFIG.cipher.enable)
    .cipher_key(CONFIG.cipher.key.clone())
    .cipher_refuse_plain(CONFIG.cipher.refuse_plain)
    .nats_address(CONFIG.nats.address.clone())
    .proxy(if CONFIG.proxy.enable_for_mesagisto {
      Some(CONFIG.proxy.address.clone())
    } else {
      None
    })
    .photo_url_resolver(|id_pair| {
      async {
        let dc_file: DcFile = serde_cbor::from_slice(&id_pair.1)?;
        Ok(dc_file.to_url())
      }
      .boxed()
    })
    .build()
    .apply()
    .await;

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
    .expect("创建Discord客户端失败");
  BOT_CLIENT.init(client.cache_and_http.clone());

  // a shutdown handle task
  let shard_manager = client.shard_manager.clone();
  tokio::spawn(async move {
    tokio::signal::ctrl_c()
      .await
      .expect("Could not register ctrl+c handler");
    info!("Mesagisto Bot 正在关闭");
    shard_manager.lock().await.shutdown_all().await;
    info!("正在保存配置文件");
    CONFIG.save();
  });

  // start to dispatch events
  // the coroutine will be suspend here until it stops
  client.start().await.expect("Client error");
  Ok(())
}
