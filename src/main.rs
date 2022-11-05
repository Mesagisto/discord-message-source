use color_eyre::eyre::Result;
use config::CONFIG;
use dashmap::DashMap;
use futures::FutureExt;
use mesagisto_client::{MesagistoConfig, MesagistoConfigBuilder};
use self_update::Status;
use serenity::{
  client::ClientBuilder, framework::standard::StandardFramework, prelude::GatewayIntents,
};
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

mod bot;
mod commands;
mod config;
pub mod ext;
mod framework;
mod handlers;
mod i18n;
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
  if !CONFIG.locale.is_empty() {
    let locale = Locale::new(&*CONFIG.locale)?;
    Locale::set_global_default(locale);
  }
  Lazy::force(&i18n::LANGUAGE_LOADER);

  if !CONFIG.enable {
    warn!("log-not-enable");
    warn!("log-not-enable-helper");
    return Ok(());
  }
  CONFIG.migrate();


  if CONFIG.auto_update.enable {
    tokio::task::spawn_blocking(|| {
      match update::update() {
        Ok(Status::UpToDate(_)) => {
          info!("log-update-check-success");
        }
        Ok(Status::Updated(_)) => {
          info!("log-upgrade-success");
          std::process::exit(0);
        }
        Err(e) => {
          tracing::error!("{}", e);
        }
      };
    })
    .await?;
  }

  MesagistoConfigBuilder::default()
    .name("dc")
    .cipher_key(CONFIG.cipher.key.clone())
    .remote_address(CONFIG.deref().centers.to_owned())
    .skip_verify(CONFIG.tls.skip_verify)
    .custom_cert(if CONFIG.tls.custom_cert.is_empty(){
      None
    }else{
      Some(CONFIG.deref().tls.custom_cert.to_owned())
    })
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
    .group(&framework::MESAGISTO_GROUP);
  // .normal_message(handlers::message_hook);

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
    .event_handler(handlers::Handler)
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
  info!("log-shutdown");
  CONFIG.save().await.expect("保存配置文件失败");
  CONFIG.save().await?;
  Ok(())
}
