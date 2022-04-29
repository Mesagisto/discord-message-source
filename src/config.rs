use arcstr::ArcStr;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[basic_derive]
#[derive(AutoConfig)]
#[location = "config/dc.yml"]
pub struct Config {
  #[educe(Default = false)]
  pub enable: bool,
  pub discord: DiscordConfig,
  pub proxy: ProxyConfig,
  pub nats: NatsConfig,
  pub cipher: CipherConfig,
  pub bindings: DashMap<u64, ArcStr>,
  target_address_mapper: DashMap<u64, ArcStr>,
}
impl Config {
  pub fn mapper(&self, target: &u64) -> Option<ArcStr> {
    match self.bindings.get(target) {
      Some(v) => return Some(v.clone()),
      None => return None,
    }
  }
  pub fn migrate(&self){
    for pair in &self.target_address_mapper {
      self.bindings.insert(pair.key().clone(), pair.value().clone());
    }
    self.target_address_mapper.clear();
  }
}

#[basic_derive]
pub struct NatsConfig {
  // pattern: "nats://{host}:{port}"
  #[educe(Default = "nats://itsusinn.site:4222")]
  pub address: ArcStr,
}

#[basic_derive]
pub struct DiscordConfig {
  #[educe(Default = "BOT.TOKEN")]
  pub token: String,
}

#[basic_derive]
pub struct ProxyConfig {
  #[educe(Default = false)]
  pub enable: bool,
  #[educe(Default = true)]
  pub enable_for_mesagisto: bool,
  // pattern: "http://{username}:{password}@{host}:{port}"
  #[educe(Default = "http://127.0.0.1:7890")]
  pub address: ArcStr,
}

#[basic_derive]
pub struct CipherConfig {
  #[educe(Default = true)]
  pub enable: bool,
  #[educe(Default = "this-is-an-example-key")]
  pub key: ArcStr,
  #[educe(Default = true)]
  pub refuse_plain: bool,
}
