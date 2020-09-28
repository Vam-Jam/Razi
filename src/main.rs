pub mod commands;
pub mod settings;

use std::{cell::RefCell, collections::HashSet};

use serenity::{
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

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is now connected!", ready.user.name);
    }

    fn guild_member_addition(&self, ctx: Context, _guild_id: GuildId, mut new_member: Member) {
        // Currently dont care about guild ID, TODO tho i swear
        // TODO-> Error checks

        let result = new_member.add_role(&ctx, RoleId(636085201461837834));

        if result.is_err() {
            println!("new_member error todo");
        }

        let gulag_date = Utc::now() - Duration::weeks(2);

        let gulaged = if new_member.user_id().created_at() > gulag_date {
            new_member
                .add_role(&ctx, RoleId(377203918557675530))
                .expect("rip role");
            ChannelId(394522201589809173).send_message(&ctx.http, |m| {
				m.content(format!("Hey {}! Your account is too new for us, so we have placed you in here.\nYou can talk here, an admin may let you out after you request.", new_member.mention()));
				m
			}).expect("rip message");

            true
        } else {
            false
        };

        let error = match result {
            Ok(_) => false,
            Err(wtf) => {
                print!("{}", wtf);
                true
            }
        };

        let _result = ChannelId(444912231176601600).send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.colour(Colour::from_rgb(52, 235, 95));

                if error {
                    e.title(
                        "New user has joined, Error in console",
                    );
                } else {
                    e.title("New user has joined!");
                }

                let url = new_member.user.read().avatar_url();
                if url.is_some() {
                    e.thumbnail(url.unwrap());
                } else {
                    e.thumbnail(new_member.user.read().default_avatar_url());
                }

                e.description(new_member.display_name());

                e.fields(vec![
                    (
                        "Creation date (UTC)",
                        format!("{}", new_member.user_id().created_at()),
                        false,
                    ),
                    ("User mention for quick access", new_member.mention(), false),
                    (
                        "Has user been gulaged",
                        (if gulaged {
                            "User has been gulaged for being under 2 weeks old"
                        } else {
                            "User has not been gulaged, account over 2 weeks old"
                        }).to_string(),
                        false,
                    ),
                ]);
                e
            });
            m
        });
    }

    fn guild_member_removal(
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

fn main() {
    let mut config = RaziConfig::new();

    RAZI_CONFIG.with(|cell| {
        config = cell.borrow().clone();
    });

    let token: &String = match &config.discord.release_run {
        true => &config.discord.release_token,
        false => &config.discord.test_token,
    };

    let mut client = Client::new(&token, Handler).expect("Error creating client");

    let (owners, _) = match client.cache_and_http.http.get_current_application_info() {
        // get owner id for a few commands
        Ok(info) => {
            let mut owners = HashSet::new();
            for owner in &config.discord.owners {
                owners.insert(UserId(*owner));
            }

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.allow_dm(false)
                    .case_insensitivity(true)
                    .prefixes(&config.discord.prefixes)
                    .owners(owners)
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
            .help(&MY_HELP),
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
#[individual_command_tip = "Commands only work in bot area, excluding the help command.
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
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}
