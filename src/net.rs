use serenity::http::HttpBuilder;
use std::sync::Arc;

use crate::config::CONFIG;

pub async fn build_http() -> serenity::http::Http {
  if CONFIG.proxy.enabled {
    let builder = reqwest::Client::builder()
      .use_rustls_tls()
      .proxy(
        reqwest::Proxy::all(CONFIG.proxy.address.clone()).expect("Failed to create reqwest::Proxy"),
      )
      .build()
      .expect("Failed to create reqwest::Client builder");
    HttpBuilder::new(CONFIG.discord.token.clone())
      .proxy(CONFIG.proxy.address.clone())
      .expect("Failed to create proxy for serenity")
      .client(Arc::new(builder))
      .await
      .expect("Failed to build serenity's  http client")
  } else {
    let builder = reqwest::Client::builder()
      .use_rustls_tls()
      .build()
      .expect("Failed to create reqwest::Client builder");
    HttpBuilder::new(CONFIG.discord.token.clone())
      .client(Arc::new(builder))
      .await
      .expect("Failed to build serenity's http client")
  }
}
