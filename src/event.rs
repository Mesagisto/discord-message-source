use serenity::{
  async_trait,
  client::{Context, EventHandler},
  model::prelude::Ready,
};

pub struct Handler;
#[async_trait]
impl EventHandler for Handler {
  async fn ready(&self, _: Context, ready: Ready) {
    log::info!("Bot:{} is connected!", ready.user.name);
  }
}
