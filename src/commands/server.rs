use crate::misc::tcpr;
use crate::razi_toml::{Config, KagServer};

use chrono::Utc;
use isahc::AsyncReadResponseExt;
use serde::Deserialize;
use serde_json::from_str as convert_from_str;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult, Delimiter},
    model::channel::Message,
    prelude::Context,
    utils::{content_safe, Colour, ContentSafeOptions},
};

use tcpr::Server;

#[command]
#[help_available]
#[aliases("s", "server")]
#[description("View server status, read pins to see current active list")]
pub async fn kag_server_status(ctx: &Context, msg: &Message) -> CommandResult {
    let config_lock = {
        let data_read = ctx.data.read().await;

        // Only thing cloned here is the reference
        data_read
            .get::<Config>()
            .expect("Expecting config in client data")
            .clone()
    };

    let server_config: Vec<KagServer>;

    {
        let config = config_lock.read().await;

        server_config = config.kag_servers.clone().unwrap_or_default();
    }

    if server_config.is_empty() {
        msg.reply(ctx, "No server's in config file.").await?;
        return Ok(());
    }

    let first_arg = {
        let mut args = Args::new(msg.content.as_str(), &[Delimiter::Single(' ')]);

        if args.len() < 2 {
            msg.reply(ctx, "Command does not have enough arguments")
                .await?;
            return Ok(());
        }

        // Args include's the initial command text, so we need to skip that
        args.advance();

        // Wont panic since we already know length will be > 1
        args.current().unwrap().to_lowercase()
    };

    let server_to_query = {
        let mut server_to_query = None;

        for server in server_config {
            if server.name == first_arg {
                server_to_query = Some(server);
                break;
            }

            if let Some(aliases) = &server.aliases {
                for alias in aliases {
                    if alias == &first_arg {
                        server_to_query = Some(server);
                        break;
                    }
                }
            }
        }

        if server_to_query.is_none() {
            msg.reply(ctx, "Could not find the server you requested.")
                .await?;
            return Ok(());
        }

        server_to_query.unwrap()
    };

    // Let the user know (and edit it once message has been recieved);
    let mut userfeedback = msg.reply(ctx, "Requesting from kag api now...").await?;

    let response = isahc::get_async(format!(
        "https://api.kag2d.com/v1/game/thd/kag/server/{}/status",
        &server_to_query.address
    ))
    .await;

    let json_text = match response {
        Ok(mut reply) => reply.text().await?,
        Err(error) => {
            userfeedback
                .edit(ctx, |f| {
                    f.content("Error getting response from api.kag2d.com. This has been logged internally.");
                    f
                })
                .await?;
            println!("User args: {}\nError: {}", msg.content, error);
            return Ok(());
        }
    };

    let json: json_kag = match convert_from_str(&json_text) {
        Ok(result) => result,
        Err(error) => {
            userfeedback
                .edit(ctx, |f| {
                    f.content("Error converting json to object. This has been logged internally.");
                    f
                })
                .await?;
            println!("User args: {}\nError: {}", msg.content, error);
            return Ok(());
        }
    };


    if let Some(server) = json.serverStatus {
        let mut player_count = server.currentPlayers;
        let name = server.name;
        let mut players = String::new();

        if player_count == 0 {
            players = String::from("No players currently in game");
        } else {
            for mut player in server.playerList {
                // Sanatize player names for discord
                player = content_safe(&ctx.cache, &player, &ContentSafeOptions::default()).await;

                player = player.replace("_", r"\_");
                player = player.replace("*", r"\*");
                player = player.replace("~", r"\~");

                players += format!("{}\n", player).as_str();
            }

            // We need to check the length, since player_count is not reliable
            // Player_count is commonly outdated, so when a server freezes we end up sending an empty string
            // Causing an embed error
            if players.is_empty() {
                players = String::from("Seems like the server is frozen 🧊");
                player_count = 0;
            }
        }

        // We have to send a new message otherwise discord wont embed image >:(
        let result = msg
            .channel_id
            .send_message(ctx, |f| {
                f.embed(|e| {
                    e.color(Colour::from_rgb(52, 235, 95));
                    e.title(name);
                    e.fields(vec![
                        ("Player count", format!("{}", player_count), false),
                        ("Players", players, false),
                    ]);
                    if server_to_query.minimap {
                        e.image(format!(
                            "https://api.kag2d.com/v1/game/thd/kag/server/{}/minimap?{}",
                            &server_to_query.address,
                            Utc::now().timestamp()
                        ));
                    }
                    e
                });
                f
            })
            .await;

        if let Err(error) = result {
            // Attempt to edit our last message
            userfeedback
                .edit(ctx, |f| {
                    f.content("Sending embed failed. This has been logged internally.");
                    f
                })
                .await?;
            println!("Embed error: {}, Json: {}", error, json_text);
            return Ok(());
        } else {
            userfeedback.delete(ctx).await?;
        }
    } else {
        userfeedback
            .edit(ctx, |f| {
                f.content(format!(
                    "api.kag2d.com has returned: \nError code: {}\nMessage: {}",
                    json.statusCode.unwrap_or_default(),
                    json.statusMessage.unwrap_or_default()
                ));
                f
            })
            .await?;
    }

    Ok(())
}

/// api.kag2d.com's json formatted struct

#[allow(non_snake_case, non_camel_case_types, dead_code)]
#[derive(Deserialize)]
struct json_kag {
    // Server status if server hasnt shat it self
    serverStatus: Option<status>,

    // Error codes if server has shat it self
    statusCode: Option<u16>,
    statusMessage: Option<String>,
    statusSubCode: Option<u16>,
    statusSubType: Option<String>,
}

#[allow(non_snake_case, non_camel_case_types, dead_code)]
#[derive(Deserialize)]
struct status {
    DNCycle: bool,
    IPv4Address: String,
    connectable: bool,
    currentPlayers: i32,
    lastUpdate: String,
    name: String,
    playerList: Vec<String>,
    port: i32,
}
