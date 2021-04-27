mod commands;
mod razi_toml;

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
        id::{ChannelId, UserId}
    },
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
impl EventHandler for Handler {}

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
