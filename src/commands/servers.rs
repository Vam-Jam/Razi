#[path = "../settings/toml.rs"] pub mod toml;

use std::{
	collections::{HashSet}, 
	cell::{RefCell }
};
use serenity::{
	model::{channel::Message, gateway::Ready, id::{UserId, ChannelId}},
	framework::standard::{Args, Delimiter, HelpOptions, CommandGroup, macros::*, CommandResult, 
		StandardFramework,  help_commands},
	prelude::*,
	utils::{
		content_safe,
		ContentSafeOptions,
		Colour
	}
};
use serde::{Deserialize};
use serde_json::from_str;
use isahc::prelude::*;
use chrono::{Utc};
use crate::RAZI_CONFIG;
use crate::settings::toml::*;


#[command]
#[help_available]
#[aliases("s","server")]
#[description("View server status, read pins to see current active list")]
pub fn server_status(ctx: &mut Context, msg: &Message) -> CommandResult {

	let mut args = Args::new(msg.content.as_str(), &[Delimiter::Single(' ')]);
	
	let mut config = RaziConfig::new();

	RAZI_CONFIG.with(|cell| {
		config = cell.borrow().clone();
	});

	let server_list = config.kag_server;
	let owner_list = config.discord.owners;

	args.advance(); // skip to first arg
	let first_arg:Option<String> = match args.single::<String>() {
		Ok(passed_arg) => Some(passed_arg),
		Err(failed_arg) => { 
			print!("Passed arg error {}", failed_arg); 
			None
		}
	};

	if first_arg.is_none() {
		msg.reply(&ctx, "Please pass a server name (check pins for current list of servers)")?;
		return Ok(());
	}

	let first_arg = first_arg.unwrap().to_lowercase();
	let mut ip = String::new();
	let mut port = String::new();
	let mut minimap = false;

	if server_list.is_none() {
		return Ok(());
	}

	for server in server_list.unwrap() {
		let mut iter = server.names.iter();

		let found: Option<&String> = iter.find(| &x | x.to_lowercase() == first_arg );

		if found.is_some() {
			ip =  String::from(server.ip.as_str()); // there's better ways to handle this, but good enough for now
			port = String::from(server.port.as_str());
			minimap = server.minimap;
			break;
		}
	}

	let ip = ip;
	let port = port;
	let minimap = minimap;

	if ip.is_empty() { 
		msg.reply(&ctx, "Server name not found, please check pins for current active list.")?;
		return Ok(());
	}

	let is_owner = owner_list.into_iter().find(|x| x == msg.author.id.as_u64()).is_some();
	let response = isahc::get(format!("https://api.kag2d.com/v1/game/thd/kag/server/{}/{}/status", &ip, &port));
	if response.is_err() {
		let err = response.err().unwrap();
		println!("{}", &err);
		if is_owner {
			msg.reply(&ctx, format!("API get request error: {}", &err))?; 
		}
		return Ok(());
	}

	let server_json: Option<kag_server> = match from_str(&response.unwrap().text()?) {
		Ok(result) => Some(result),
		Err(errmsg) => {
			println!("{}", &errmsg); 
			if is_owner {
				msg.reply(&ctx, format!("Json error: {}", &errmsg))?;
			}
			None
		}
	};

	if server_json.is_none() {
		return Ok(())
	}
	let server_json = server_json.unwrap();

	// Message builder time
	let server_name = server_json.serverStatus.name;
	let player_count = server_json.serverStatus.currentPlayers;
	let mut players = String::new();
	
	if &player_count == &0 {
		players = String::from("No players currently in game");
	} else {
		for mut player in server_json.serverStatus.playerList {
			player = content_safe(&ctx.cache, &player, &ContentSafeOptions::default());		
			players += format!("{}\n", player).as_str();
		}
	}

	let result = msg.channel_id.send_message(&ctx.http, |m| {
		m.embed(|e| {
			e.colour(Colour::from_rgb(52, 235, 95));
			e.title(server_name);
			e.fields(vec![
                ("Player count", format!("{}",player_count),false),
                ("Players", players,false),
			]);
			
			if minimap {
				e.image(format!("https://api.kag2d.com/v1/game/thd/kag/server/{}/{}/minimap?{}", &ip, &port, Utc::now().timestamp()));
			}
			e
		});
		m
	});

	if result.is_err() {
		let errmsg = result.err().unwrap();
		println!("{}",errmsg);
		return Ok(());
	}

	Ok(())
}

#[allow(non_snake_case, non_camel_case_types, dead_code)]
#[derive(Deserialize)]
struct kag_server{
    serverStatus: status,
}

#[allow(non_snake_case, non_camel_case_types, dead_code)]
#[derive(Deserialize)]
struct status{
    DNCycle: bool,
    IPv4Address: String,
    connectable: bool,
    currentPlayers: i32,
    lastUpdate: String,
    name: String,
    playerList: Vec<String>,
    port: i32,
}