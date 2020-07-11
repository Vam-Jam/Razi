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

pub mod toml; // discord settings

thread_local!(static RAZI_CONFIG: RefCell<toml::RaziConfig> = RefCell::new(toml::RaziConfig::get_config()));

#[group]
#[commands(info)]
struct General;

#[group]
#[commands(server_request)]
struct Api;

struct Handler;

impl EventHandler for Handler {
	fn ready(&self, _:Context, ready: Ready){
		println!("{} is now connected!", ready.user.name);
	}
}

fn main() {
	let mut config = toml::RaziConfig::new();

	RAZI_CONFIG.with(|cell| {
		config = cell.borrow().clone();
	});
	
	let mut client = Client::new(&config.discord.token, Handler).expect("Error creating client");

	let (owners, _) = match client.cache_and_http.http.get_current_application_info() { // get owner id for a few commands
        Ok(info) => {
            let mut owners = HashSet::new();
			for owner in &config.discord.owners {
				owners.insert(UserId(*owner));
			}

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
	};
	
	client.with_framework(StandardFramework::new()
		.configure(|c| c
			.allow_dm(false)
			.case_insensitivity(true)
			.prefixes(&config.discord.prefixes)
			.owners(owners)
			.allowed_channels( { 
				let mut allowed_channels: HashSet<ChannelId> = HashSet::new();
				
				for channels in &config.discord.allowed_channels {
					allowed_channels.insert(ChannelId(channels.clone()));
				}

				allowed_channels
			}))
		.group(&GENERAL_GROUP)
		.group(&API_GROUP)
		.help(&MY_HELP)

		
	);


	if let Err(why) = client.start() {
		println!("Client error: {:?}", why);
	}
}



/// ///////////////////
/// Commands sit below
/// 

#[command]
#[help_available]
#[description("About me and source code")]
fn info(ctx: &mut Context, msg: &Message) -> CommandResult {
	let msg = msg.channel_id.send_message(&ctx.http, |m| {
		m.embed(|e| {
			e.title("Purpose of this bot");
			e.description("Source code can be found here: <https://github.com/Vam-Jam/Razi>");
			e.thumbnail("https://cdn.discordapp.com/attachments/551770125578010624/726435452525084672/hackerman.jpg");
			e.fields(vec![
				("Why another bot?", "bored lol", false),
				("Why is it named Razi?", 
					"Riza's source code kinda sucked, 0 error handling, and just general weird layout.\nRazi is just a better version of Riza\nThis bot however is being made ground up with ease of use.", false),
				("Can i suggest stuff?", "Sure, ping me and ill add it to <https://trello.com/b/rdklywLp/razi> if i think its do-able and suitable", false),
			]);
			e
		});
		m
	});

	if let Err(why) = msg {
		println!("Error sending info:\n{:?}", why);
	}

	Ok(())
}


#[help]
#[individual_command_tip =
"Commands only work in bot area, excluding the help command.
If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Strike"]
#[wrong_channel = "Hide"]
#[no_help_available_text = "**Error**: Please use this command in bot area"]
fn my_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}


#[command]
#[help_available]
#[aliases("s","server")]
#[description("View server status, read pins to see current active list")]
fn server_request(ctx: &mut Context, msg: &Message) -> CommandResult {

	let mut args = Args::new(msg.content.as_str(), &[Delimiter::Single(' ')]);
	
	let mut config = toml::RaziConfig::new();

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

	for server in server_list {
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
