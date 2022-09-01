#![allow(incomplete_features)]
#![feature(capture_disjoint_fields)]

use color_eyre::eyre::Result;
use config::CONFIG;
use dashmap::DashMap;
use mesagisto_client::{MesagistoConfig, MesagistoConfigBuilder};
use self_update::Status;
use serenity::{
  client::ClientBuilder, framework::standard::StandardFramework, prelude::GatewayIntents,
};
use smol::future::FutureExt;
use tracing::{info, warn};

use crate::{
  bot::BOT_CLIENT,
  config::Config,
  handlers::{receive, receive::packet_handler},
};

#[macro_use]
extern crate educe;
#[macro_use]
extern crate automatic_config;
#[macro_use]
extern crate singleton;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate rust_i18n;
i18n!("locales");

mod bot;
mod commands;
mod config;
mod event;
pub mod ext;
mod framework;
mod handlers;
mod log;
mod net;
mod update;

#[tokio::main]
async fn main() -> Result<()> {
  if cfg!(feature = "color") {
    color_eyre::install()?;
  } else {
    color_eyre::config::HookBuilder::new()
      .theme(color_eyre::config::Theme::new())
      .install()?;
  }

  self::log::init();
  run().await?;
  Ok(())
}

async fn run() -> Result<()> {
  Config::reload().await?;
  if !&CONFIG.locale.is_empty() {
    rust_i18n::set_locale(&CONFIG.locale);
  } else {
    use sys_locale::get_locale;
    let locale = get_locale()
      .unwrap_or_else(|| String::from("en-US"))
      .replace('_', "-");
    rust_i18n::set_locale(&locale);
    info!("{}", t!("log.locale-not-configured", locale_ = &locale));
  }
  if !CONFIG.enable {
    warn!("{}", t!("log.not-enable"));
    warn!("{}", t!("log.not-enable-helper"));
    return Ok(());
  }
  CONFIG.migrate();
  if cfg!(feature = "beta") {
    std::env::set_var("GH_PRE_RELEASE", "1");
    std::env::set_var("BYPASS_CHECK", "1");
  }

  if CONFIG.auto_update.enable {
    tokio::task::spawn_blocking(|| {
      match update::update() {
        Ok(Status::UpToDate(_)) => {
          info!("{}", t!("log.update-check-success"));
        }
        Ok(Status::Updated(_)) => {
          info!("{}", t!("log.upgrade-success"));
          std::process::exit(0);
        }
        Err(e) => {
          error!("{}", e);
        }
      };
    })
    .await?;
  }
  let remotes = DashMap::new();
  remotes.insert(
    arcstr::literal!("mesagisto"),
    "msgist://center.itsusinn.site:6996".into(),
  );
  MesagistoConfigBuilder::default()
    .name("dc")
    .cipher_key(CONFIG.cipher.key.clone())
    .local_address("0.0.0.0:0")
    .remote_address(remotes)
    .proxy(if CONFIG.proxy.enable {
      Some(CONFIG.proxy.address.clone())
    } else {
      None
    })
    .build()?
    .apply()
    .await?;
  MesagistoConfig::packet_handler(|pkt| async { packet_handler(pkt).await }.boxed());

  let framework = StandardFramework::new()
    .configure(|c| c.prefix("/"))
    .help(&framework::HELP)
    .group(&framework::MESAGISTO_GROUP)
    .normal_message(handlers::message_hook);

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
  let mut client = ClientBuilder::new_with_http(http, intents)
    .event_handler(event::Handler)
    .framework(framework)
    .await?;
  BOT_CLIENT.init(client.cache_and_http.clone());

  let shard_manager = client.shard_manager.clone();
  receive::recover().await?;
  tokio::spawn(async move {
    client.start().await.expect("Client error");
  });

  tokio::signal::ctrl_c().await?;
  shard_manager.lock().await.shutdown_all().await;
  info!("Mesagisto信使 正在关闭");
  info!("正在保存配置文件");
  CONFIG.save().await?;
  Ok(())
}
