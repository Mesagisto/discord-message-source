use std::{ops::Deref, sync::Arc};

use mesagisto_client::LateInit;
use serenity::CacheAndHttp;

#[derive(Singleton, Default)]
pub struct BotClient {
  inner: LateInit<Arc<CacheAndHttp>>,
}
impl BotClient {
  pub fn init(&self, bot: Arc<CacheAndHttp>) {
    self.inner.init(bot)
  }
}
impl Deref for BotClient {
  type Target = Arc<CacheAndHttp>;
  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}
