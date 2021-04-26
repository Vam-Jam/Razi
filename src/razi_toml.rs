use serde::Deserialize;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;
use std::{default::Default, fs::File, io::prelude::Read};
use tokio::sync::RwLock;
use toml::from_str;

#[derive(Deserialize, Default, Debug)]
pub struct Config {
    pub discord: Discord,
    pub kag_servers: Option<Vec<KagServer>>,
}

#[derive(Deserialize, Default, Debug)]
pub struct Discord {
    pub token: String,
    pub prefix: String,
    pub owners: Option<Vec<u64>>,
    pub admin_roles: Option<Vec<u64>>,
    pub bot_channels: Option<Vec<u64>>,
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct KagServer {
    pub name: String,
    pub aliases: Option<Vec<String>>,
    pub minimap: bool,
    pub address: String,
}

// Required for us to write to serenity's client data
impl TypeMapKey for Config {
    type Value = Arc<RwLock<Config>>;
}

impl Config {
    // Panic if we can't read token (and other important info)
    pub fn read_from_file() -> Config {
        let mut config = Config::default();
        config.load_config();

        config
    }

    pub fn load_config(&mut self) {
        let mut config = String::new();

        // TODO: Generate file if Razi is not found. Will panic for now
        let mut file = File::open("./Razi.toml").unwrap();

        file.read_to_string(&mut config).unwrap_or_default();

        let config: Config = from_str(config.as_str()).unwrap();

        // TODO: Find a better way to do this, could not find a way to
        // replace self with the new Config ( rust noob :p )
        self.discord = config.discord;
        self.kag_servers = config.kag_servers;
    }
}
