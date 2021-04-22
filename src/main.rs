mod commands;
mod razi_toml;

use serenity::{
    async_trait,
    client::{bridge::gateway::GatewayIntents, Client, EventHandler},
    framework::standard::{macros::group, StandardFramework},
    model::id::{ChannelId, UserId},
};
use std::collections::HashSet;

use tokio::sync::RwLock;

use std::sync::Arc;

use commands::basic::*;

use razi_toml::Config;

#[group]
#[commands(ping, reload_config)]
struct General;

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
        .group(&GENERAL_GROUP);


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
