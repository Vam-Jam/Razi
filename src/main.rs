use std::{collections::{HashSet}, cell::{RefCell, RefMut, Ref}};
use serenity::{
	model::{channel::Message, gateway::Ready, id::{UserId, ChannelId}},
	framework::standard::{Args, HelpOptions, CommandGroup, macros::*, CommandResult, 
		StandardFramework, DispatchError::BlockedChannel, help_commands},
	prelude::*,
};

pub mod toml; // discord settings

thread_local!(static RaziConfig: RefCell<toml::RaziConfig> = RefCell::new(toml::RaziConfig::get_config()));

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

	RaziConfig.with(|cell| {
		config = cell.borrow().clone();

	});
	
	let mut client = Client::new(&config.discord.token, Handler).expect("Error creating client");

	let (owners, bot_id) = match client.cache_and_http.http.get_current_application_info() { // get owner id for a few commands
        Ok(info) => {
            let mut owners = HashSet::new();
			owners.insert(info.owner.id);
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
			.prefix(&config.discord.prefix.as_str())
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
#[description("View server status, read pins to see current active list")]
#[owners_only(true)]
fn server_request(ctx: &mut Context, msg: &Message) -> CommandResult {
	let raziconfig = toml::RaziConfig::get_config(); // todo, cache and add a command to hot reload
	let list = raziconfig.kag_server;

	for server in list {
		println!("{}", server.ip);
	}


	Ok(())
}