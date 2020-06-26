use toml::from_str;
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;


// TODO: Add other parameters to toml
// TODO: Make file if it doesnt exist

#[derive(Deserialize)]
struct RaziConfig {
	discord: Discord,
}

#[derive(Deserialize)]
struct Discord {
	token: String,
}

fn main() {
	println!("{}",get_discord_token());
}


fn get_discord_token() -> String { // Get token from toml

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

	let config = config.unwrap();
	config.discord.token
}
