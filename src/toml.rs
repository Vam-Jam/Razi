use toml::from_str;
use serde::{Deserialize};
use std::{
	clone::Clone,
	default::Default,
	fs::File,
	io::prelude::Read,
};

// shamelessly stolen from stackoverflow because i dont fully understand macro's
macro_rules! pub_struct {
    ($name:ident {$($field:ident: $t:ty,)*}) => {
        #[derive(Deserialize, Clone, Default)]
        pub struct $name {
            $(pub $field: $t),*
        }
    }
}

pub_struct!(RaziConfig {
	discord: Discord,
	kag_server: Vec<KagServer>,
});

pub_struct!(Discord {
	token: String,
	prefix: String,
	allowed_channels: Vec<u64>,
	owners: Vec<u64>,
});

pub_struct!(KagServer {
	names: Vec<String>,
	ip: String,
	port: String,
	minimap: bool,
});

impl RaziConfig {
	pub fn new() -> RaziConfig {
		RaziConfig::default()
	}
	
	pub fn get_config() -> RaziConfig { // Get token from toml
		RaziConfig::get_razi_config()
	}

	fn get_razi_config() -> RaziConfig {
		let mut toml_file = match File::open("./Razi.toml") {
			Ok(file) => file,
			Err(_) => {
				panic!("File could not be found");
			}
		};

		let mut config = String::new();

		match toml_file.read_to_string(&mut config) {
			Ok(_) => (), 
			Err(error) => panic!("File could not be read! {:?}", error),
		}

		let config: Option<RaziConfig> = match from_str(config.as_str()) {
			Ok(login) => Some(login),
			Err(error) => {
				println!("Couldnt convert to toml! {:?}", error);
				None
			}
		};

		if config.is_none() {
			panic!("Exiting due to toml conversion failure");
		} 

		config.unwrap()
	}
}
