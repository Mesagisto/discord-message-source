use std::sync::Arc;

use anyhow::Result;
use config::CONFIG;
use message::init_nc;
use once_cell::sync::OnceCell;
use serenity::{CacheAndHttp, client::{ClientBuilder, bridge::gateway::GatewayIntents}, framework::standard::StandardFramework};

#[macro_use]
extern crate log;
#[macro_use]
extern crate educe;

mod config;
mod net;
mod commands;
mod message;
mod data;
mod event;
mod framework;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::formatted_builder()
        .write_style(pretty_env_logger::env_logger::WriteStyle::Auto)
        .filter(Some("discord_mesaga_fonto"), log::LevelFilter::Info)
        .filter(Some("serenity"), log::LevelFilter::Warn)
        .init();
    run().await?;
    Ok(())
}
static BOT_CLIENT:OnceCell<Arc<CacheAndHttp>> = OnceCell::new();
pub fn get_bot_client() -> Arc<CacheAndHttp> {
    BOT_CLIENT.get().expect("BOT_CLIENT is not initialized").clone()
}

async fn run() -> Result<(), anyhow::Error>{
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
        .await.expect("Err creating client");
    let _ = BOT_CLIENT.set(client.cache_and_http.clone());
    // a shutdown handle task
    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
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
