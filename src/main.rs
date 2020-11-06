pub mod commands;
pub mod settings;

use std::{cell::RefCell, collections::HashSet};

use serenity::{
    async_trait,
    client::bridge::gateway::GatewayIntents,
    framework::standard::{
        help_commands, macros::*, Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
    },
    model::{
        channel::Message,
        gateway::Ready,
        guild::Member,
        id::{ChannelId, GuildId, RoleId, UserId},
        user::User,
    },
    prelude::{Client, Context, EventHandler, Mentionable},
    utils::Colour,
};

use chrono::{Duration, Utc};
use commands::servers::*;

use settings::toml::*;

thread_local!(pub static RAZI_CONFIG: RefCell<RaziConfig> = RefCell::new(RaziConfig::get_config()));

#[group]
#[commands(info)]
struct General;

#[group]
#[commands(server_status)]
struct Api;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is now connected!", ready.user.name);
    }

    async fn guild_member_addition(&self, ctx: Context, _guild_id: GuildId, new_member: Member) {
        let mut error = false; // this will get toggled to true if there's an error

        let mut new_member = new_member;
        let result = new_member.add_role(&ctx, RoleId(636085201461837834)).await;

        if result.is_err() {
            println!("new_member error {}", &result.err().unwrap());
            error = true;
        }

        let gulag_date = Utc::now() - Duration::weeks(2);

        let gulaged = if new_member.user.id.created_at() > gulag_date {
            let result = new_member.add_role(&ctx, RoleId(377203918557675530)).await;

            if result.is_err() {
                println!("added role error {}", result.err().unwrap());
                error = true;
            }

            let result = ChannelId(394522201589809173).send_message(&ctx.http, |m| {
				m.content(format!("Hey {}! Your account is too new for us, so we have placed you in here.\nYou can talk here, an admin may let you out after you request.", new_member.mention()));
				m
            }).await;

            if result.is_err() {
                println!("new user gulag message error {}", result.err().unwrap());
                error = true;
            }

            true
        } else {
            false
        };

        let _result = ChannelId(444912231176601600)
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.colour(Colour::from_rgb(52, 235, 95));

                    if error {
                        e.title("New user has joined, Error in console");
                    } else {
                        e.title("New user has joined!");
                    }

                    let url = new_member.user.avatar_url();
                    if url.is_some() {
                        e.thumbnail(url.unwrap());
                    } else {
                        e.thumbnail(new_member.user.default_avatar_url());
                    }

                    e.description(new_member.display_name());

                    e.fields(vec![
                        (
                            "Creation date (UTC)",
                            format!("{}", new_member.user.id.created_at()),
                            false,
                        ),
                        ("User mention for quick access", new_member.mention(), false),
                        (
                            "Has user been gulaged",
                            (if gulaged {
                                "User has been gulaged for being under 2 weeks old"
                            } else {
                                "User has not been gulaged, account over 2 weeks old"
                            })
                            .to_string(),
                            false,
                        ),
                    ]);
                    e
                });
                m
            })
            .await;

        if _result.is_err() {
            println!("{}", _result.err().unwrap());
        }
    }

    async fn guild_member_removal(
        &self,
        ctx: Context,
        _guild: GuildId,
        user: User,
        _member_data_if_available: Option<Member>,
    ) {
        let _result = ChannelId(444912231176601600).send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.colour(Colour::from_rgb(230, 10, 10));

                e.title("User has left the game!");

                if let Some(url) = user.avatar_url() {
                    e.thumbnail(url);
                } else {
                    e.thumbnail(user.default_avatar_url());
                }

                e.description(user.name);

                e.fields(vec![
                    (
                        "Creation date (UTC)",
                        format!("{}", user.id.created_at()),
                        false,
                    ),
                    ("User mention for quick access", user.id.mention(), false),
                ]);
                e
            });
            m
        });
    }
}

#[tokio::main]
async fn main() {
    let mut config = RaziConfig::new();

    RAZI_CONFIG.with(|cell| {
        config = cell.borrow().clone();
    });

    let token: &String = match &config.discord.release_run {
        true => &config.discord.release_token,
        false => &config.discord.test_token,
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.allow_dm(false)
                .case_insensitivity(true)
                .prefixes(&config.discord.prefixes)
                .owners({
                    let mut owners = HashSet::new();
                    for owner in &config.discord.owners {
                        owners.insert(UserId(*owner));
                    }

                    owners
                })
                .allowed_channels({
                    let mut allowed_channels: HashSet<ChannelId> = HashSet::new();

                    for channels in &config.discord.allowed_channels {
                        allowed_channels.insert(ChannelId(*channels));
                    }

                    allowed_channels
                })
        })
        .group(&GENERAL_GROUP)
        .group(&API_GROUP)
        .help(&MY_HELP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .add_intent(GatewayIntents::GUILD_MEMBERS)
        .add_intent(GatewayIntents::GUILD_MESSAGES)
        .await
        .expect("Error creating client!");

    /*let (owners, _) = match client.cache_and_http.http.get_current_application_info().await {
        // get owner id for a few commands
        Ok(info) => {
            let mut owners = HashSet::new();
            for owner in &config.discord.owners {
                owners.insert(UserId(*owner));
            }

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };*/

    //client

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

/// ///////////////////
/// Commands sit below
///

#[command]
#[help_available]
#[description("About me and source code")]
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
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

    if let Err(why) = msg.await {
        println!("Error sending info:\n{:?}", why);
    }

    Ok(())
}

#[help]
#[individual_command_tip = "Commands only work in bot area, excluding the help command.
If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Strike"]
#[wrong_channel = "Hide"]
#[no_help_available_text = "**Error**: Please use this command in bot area"]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}
