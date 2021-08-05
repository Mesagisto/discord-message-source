use log::error;
use serde_derive::{Deserialize, Serialize};
use std::{fs, io, path::Path, sync::Arc};
use once_cell::sync::Lazy;
use dashmap::DashMap;
use thiserror::Error;
use serde_yaml as yaml;

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let path = Path::new("config.yml");
    let config = read_or_create_config(path).unwrap();
    config
});

#[derive(Debug, Error)]
pub enum Error {
    #[error("无法序列化")]
    SerializationError,
    #[error("I/O error")]
    IO(#[from] io::Error),
}

#[derive(Debug, Serialize, Deserialize,Educe)]
#[educe(Default)]
pub struct Config{
    #[educe(Default = false)]
    pub enabled: bool,
    pub forwarding: ForwardingConfig,
    pub discord: DiscordConfig,
    pub proxy: ProxyConfig,
    pub target_address_mapper: DashMap<Arc<String>, Arc<String>>,
}
#[derive(Debug, Serialize, Deserialize,Educe)]
#[educe(Default)] 
pub struct ForwardingConfig {
    // pattern: "nats://{host}:{port}"
    #[educe(Default = "nats://itsusinn.site:4222")]
    pub address: String
}
#[derive(Debug, Serialize, Deserialize,Educe)]
#[educe(Default)]
pub struct DiscordConfig {
    #[educe(Default = "BOT.TOKEN")]
    pub token: String
}
#[derive(Debug, Serialize, Deserialize,Educe)]
#[educe(Default)]
pub struct ProxyConfig {
    #[educe(Default = false)]
    pub enabled: bool,
    // pattern: "http://{username}:{password}@{host}:{port}"
    #[educe(Default = "http://127.0.0.1:7890")]
    pub address: String,
}

impl Config {
    pub fn default_string() -> Result<String, Error> {
        let result = yaml::to_string(&Config::default()).map_err(|_| Error::SerializationError)?;
        Ok(result)
    }
    pub fn save(&self) {
        let ser = yaml::to_string(self).unwrap();
        log::info!("Saving the configuration file");
        fs::write("config.yml", ser).unwrap();
    }
}

fn read_or_create_config(path: &Path) -> Result<Config, Error> {
    if !path.exists() {
        fs::create_dir_all(path.parent().unwrap_or(Path::new("./")))?;
        fs::write(path, Config::default_string()?)?;
    };
    let data = fs::read(path)?;
    let result: Result<Config, yaml::Error> = yaml::from_slice(&data);
    let result = match result {
        Ok(val) => val,
        Err(_) => {
            error!("Cannot de-serialize the configuration file");
            error!("It may be caused by incompatible configuration files due to version updates");
            error!("The original file has been changed to config.toml.old, please merge the configuration files manually");
            let default_string = Config::default_string()?;
            let reanme_path = format!("{}.old", path.clone().to_string_lossy());
            let rename_path =Path::new(&reanme_path);
            fs::rename(path, rename_path)?;
            fs::write(path, default_string)?;
            Config::default()
        }
    };
    Ok(result)
}