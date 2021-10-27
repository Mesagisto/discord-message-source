use std::{path::Path, sync::Arc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

#[basic_derive]
#[derive(AutoConfig)]
#[location = "config/dc.yml"]
pub struct Config{
    #[educe(Default = false)]
    pub enabled: bool,
    pub forwarding: ForwardingConfig,
    pub discord: DiscordConfig,
    pub proxy: ProxyConfig,
    pub target_address_mapper: DashMap<Arc<String>, Arc<String>>,
}

#[basic_derive]
pub struct ForwardingConfig {
    // pattern: "nats://{host}:{port}"
    #[educe(Default = "nats://itsusinn.site:4222")]
    pub address: String
}

#[basic_derive]
pub struct DiscordConfig {
    #[educe(Default = "BOT.TOKEN")]
    pub token: String
}

#[basic_derive]
pub struct ProxyConfig {
    #[educe(Default = false)]
    pub enabled: bool,
    // pattern: "http://{username}:{password}@{host}:{port}"
    #[educe(Default = "http://127.0.0.1:7890")]
    pub address: String,
}
