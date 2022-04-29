#![allow(incomplete_features)]
#![feature(capture_disjoint_fields)]

use crate::bot::{DcFile, BOT_CLIENT};
use crate::message::handlers::receive;
use anyhow::Result;
use config::CONFIG;
use mesagisto_client::MesagistoConfig;
use serenity::{
  client::{ClientBuilder}, framework::standard::StandardFramework, prelude::GatewayIntents,
};
use smol::future::FutureExt;
use tracing::{warn, info};

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

#[tokio::main]
async fn main(){
  run().await.unwrap();
}

async fn run() -> Result<(), anyhow::Error> {

  let env = tracing_subscriber::EnvFilter::from("warn")
    .add_directive("serenity=warn".parse()?)
    .add_directive("discord_message_source=info".parse()?)
    .add_directive("mesagisto_client=trace".parse()?);
  tracing_subscriber::fmt().with_env_filter(env).init();

  if !CONFIG.enable {
    warn!("Mesagisto-Bot is not enabled and is about to exit the program.");
    warn!("To enable it, please modify the configuration file.");
    warn!("Mesagisto-Bot未被启用, 即将退出程序。");
    warn!("若要启用，请修改配置文件。");
    return Ok(());
  }
  info!("Mesagisto-Bot is starting up");
  info!("Mesagisto-Bot正在启动");
  CONFIG.migrate();
  MesagistoConfig::builder()
    .name("dc")
    .cipher_enable(CONFIG.cipher.enable)
    .cipher_key(CONFIG.cipher.key.clone())
    .cipher_refuse_plain(CONFIG.cipher.refuse_plain)
    .nats_address(CONFIG.nats.address.clone())
    .proxy(if CONFIG.proxy.enable && CONFIG.proxy.enable_for_mesagisto {
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
  let intents = {
    let mut intents = GatewayIntents::all();
    // intents.remove(GatewayIntents::GUILD_PRESENCES);
    intents.remove(GatewayIntents::DIRECT_MESSAGE_TYPING);
    intents.remove(GatewayIntents::DIRECT_MESSAGE_REACTIONS);
    intents.remove(GatewayIntents::GUILD_MESSAGE_TYPING);
    intents.remove(GatewayIntents::GUILD_MESSAGE_REACTIONS);
    intents
  };
  let mut client = ClientBuilder::new_with_http(http,intents)
    .event_handler(event::Handler)
    .framework(framework)
    .await
    .expect("创建Discord客户端失败");
  BOT_CLIENT.init(client.cache_and_http.clone());

  // a shutdown handle task
  let shard_manager = client.shard_manager.clone();
  tokio::spawn(async move {
    tokio::signal::ctrl_c()
      .await
      .expect("无法注册 Ctrl+C 处理回调");
    info!("Mesagisto Bot 正在关闭");
    shard_manager.lock().await.shutdown_all().await;
    info!("正在保存配置文件");
    CONFIG.save();
  });

  receive::recover().await?;
  // start to dispatch events
  // the coroutine will be suspend here until it stops
  client.start().await.expect("Client error");
  Ok(())
}
