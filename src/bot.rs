use std::{ops::Deref, sync::Arc};

use arcstr::ArcStr;
use mesagisto_client::{cache::CACHE};
use serde::{Deserialize, Serialize};
use serenity::CacheAndHttp;
use lateinit::LateInit;

#[derive(Debug, Serialize, Deserialize)]
pub struct DcFile(u64, u64, ArcStr);
impl DcFile {
  pub fn new(channel: &u64, attachment: &u64, fname: &ArcStr) -> Self {
    Self(*channel, *attachment, fname.clone())
  }
  pub fn to_url(&self) -> ArcStr {
    format!(
      "https://cdn.discordapp.com/attachments/{}/{}/{}",
      self.0, self.1, self.2
    )
    .into()
  }
}

#[derive(Singleton, Default)]
pub struct BotClient {
  inner: LateInit<Arc<CacheAndHttp>>,
}
impl BotClient {
  pub fn init(&self, bot: Arc<CacheAndHttp>) {
    self.inner.init(bot)
  }
  pub async fn download_file(&self, dc_file: &DcFile) -> anyhow::Result<()> {
    let url = dc_file.to_url();
    CACHE
      .file_by_url(&dc_file.1.to_be_bytes().to_vec(), &url.into())
      .await?;
    Ok(())
  }
}
impl Deref for BotClient {
  type Target = Arc<serenity::http::Http>;
  fn deref(&self) -> &Self::Target {
    &self.inner.http
  }
}
