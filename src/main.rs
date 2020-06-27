use serenity::{
	model::{channel::Message, gateway::Ready},
	framework::standard::{macros::*, CommandResult, StandardFramework},
	prelude::*,
};

pub mod toml; // discord settings

#[group]
#[commands(info)]
struct General;

struct Handler;

impl EventHandler for Handler {
	fn ready(&self, _:Context, ready: Ready){
		println!("{} is now connected!", ready.user.name);
	}
}

fn main() {
	let mut client = Client::new(toml::get_discord_token(), Handler).expect("Error creating client");

	client.with_framework(StandardFramework::new()
		.configure(|c| c.prefix("~"))
		.group(&GENERAL_GROUP));

	if let Err(why) = client.start() {
		println!("Client error: {:?}", why);
	}
}



/// ///////////////////
/// Commands sit below
/// 

#[command]
fn info(ctx: &mut Context, msg: &Message) -> CommandResult {
	let msg = msg.channel_id.send_message(&ctx.http, |m| {
		m.embed(|e| {
			e.title("Purpose of this bot");
			e.description("Source code can be found here: <https://github.com/Vam-Jam/Razi>");
			e.thumbnail("https://cdn.discordapp.com/attachments/551770125578010624/726435452525084672/hackerman.jpg");
			e.fields(vec![
				("Why another bot?", "bored lol", false),
				("Why is it named Razi?", 
					"Riza's source code kinda sucked, 0 error handling, and just general weird layout.\nThis bot however is being made ground up with ease of use.", false),
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

