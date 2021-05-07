mod commands;
mod misc;
mod razi_toml;

use chrono::{Duration, Utc};
use serenity::{
    async_trait,
    client::{bridge::gateway::GatewayIntents, Client, Context, EventHandler},
    framework::standard::{
        help_commands,
        macros::{group, help},
        Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
    },
    model::{
        channel::Message,
        gateway::Ready,
        guild::Member,
        id::{ChannelId, GuildId, RoleId, UserId},
        prelude::User,
    },
    prelude::Mentionable,
    utils::Colour,
};
use std::collections::HashSet;

use tokio::sync::RwLock;

use std::sync::Arc;

use commands::{admin::*, basic::*, server::*};

use razi_toml::Config;

#[group]
#[commands(ping, reload_config)]
struct General;

#[group]
#[commands(kag_server_status)]
struct Server;

#[group]
#[commands(restart_tc)]
struct Admin;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is now connected!", ready.user.name);
    }

    async fn guild_member_addition(
        &self,
        ctx: Context,
        _guild_id: GuildId,
        mut new_member: Member,
    ) {
        // Will error if we dont have permission or discord api issues
        if new_member
            .add_role(&ctx, RoleId(636085201461837834))
            .await
            .is_err()
        {
            println!("Could not whitelist user: {:?}", new_member);
            return;
        }

        let gulag_date = Utc::now() - Duration::weeks(2);

        // Is this user's account less then 2 weeks old
        let gulaged = if new_member.user.id.created_at() > gulag_date {
            if new_member
                .add_role(&ctx, RoleId(377203918557675530))
                .await
                .is_err()
            {
                println!("Could not whitelist user: {:?}", new_member);
                return;
            }

            let result = ChannelId(394522201589809173).send_message(&ctx.http, |m| {
                m.content(format!("Hey {}! Your account is too new for us, so we have placed you in here.\nYou can talk here, an admin may let you out after you request.", new_member.mention()));
                m
            }).await;

            if result.is_err() {
                println!(
                    "Could not notify user why they were caged: {}",
                    result.unwrap_err()
                );
            }

            true
        } else {
            false
        };

        let result = ChannelId(444912231176601600)
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.colour(Colour::from_rgb(52, 235, 95));

                    e.title("New user has joined!");

                    if let Some(url) = new_member.user.avatar_url() {
                        e.thumbnail(url);
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
                        (
                            "User mention for quick access",
                            format!("{}", new_member.mention()),
                            false,
                        ),
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

        if result.is_err() {
            println!("Logging new user joined error: {}", result.unwrap_err());
        }
    }

    async fn guild_member_removal(
        &self,
        ctx: Context,
        _guild: GuildId,
        user: User,
        _member_data_if_available: Option<Member>,
    ) {
        let result = ChannelId(444912231176601600)
            .send_message(&ctx.http, |m| {
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
                        (
                            "User mention for quick access",
                            format!("{}", user.id.mention()),
                            false,
                        ),
                    ]);
                    e
                });
                m
            })
            .await;

        if result.is_err() {
            println!("Could not log user leaving: {}", result.unwrap_err());
        }
    }
}

#[tokio::main]
async fn main() {
    let config = Config::read_from_file();

    let token = &config.discord.token;
    let owners_list = &config.discord.owners;
    let bot_channels = &config.discord.bot_channels;

    let framework = StandardFramework::new()
        .configure(|c| {
            c.owners({
                let mut owners = HashSet::new();

                if let Some(owner_list) = owners_list {
                    for owner in owner_list {
                        owners.insert(UserId(*owner));
                    }
                }

                owners
            })
            .prefix(".")
            .allowed_channels({
                let mut allowed_channels: HashSet<ChannelId> = HashSet::new();

                if let Some(bot_channels) = bot_channels {
                    for channels in bot_channels {
                        allowed_channels.insert(ChannelId(*channels));
                    }
                }

                allowed_channels
            })
        })
        .group(&GENERAL_GROUP)
        .group(&SERVER_GROUP)
        .group(&ADMIN_GROUP)
        .help(&MY_HELP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .intents(GatewayIntents::GUILD_MEMBERS | GatewayIntents::GUILD_MESSAGES)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;

        data.insert::<Config>(Arc::new(RwLock::new(config)));
    }

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
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
#[no_help_available_text = "**Error**: Command not found"]
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
