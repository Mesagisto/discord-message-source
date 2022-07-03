use arcstr::ArcStr;
use dashmap::DashMap;

#[config_derive]
#[derive(AutomaticConfig)]
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
    self.bindings.get(target).map(|v| v.clone())
  }
  pub fn migrate(&self){
    for pair in &self.target_address_mapper {
      self.bindings.insert(*pair.key(), pair.value().clone());
    }
    self.target_address_mapper.clear();
  }
}

#[config_derive]
pub struct NatsConfig {
  // pattern: "nats://{host}:{port}"
  #[educe(Default = "nats://itsusinn.site:4222")]
  pub address: ArcStr,
}

#[config_derive]
pub struct DiscordConfig {
  #[educe(Default = "BOT.TOKEN")]
  pub token: String,
}

#[config_derive]
pub struct ProxyConfig {
  #[educe(Default = false)]
  pub enable: bool,
  // pattern: "http://{username}:{password}@{host}:{port}"
  #[educe(Default = "http://127.0.0.1:7890")]
  pub address: ArcStr,
}

#[config_derive]
pub struct CipherConfig {
  #[educe(Default = "default")]
  pub key: ArcStr,
}
