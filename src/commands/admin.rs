use crate::razi_toml::{Config, Discord};
use serenity::{
    framework::standard::{
        macros::{check, command},
        Args, CommandOptions, CommandResult, Reason,
    },
    model::channel::Message,
    prelude::Context,
};

use std::process::Command as cmd;

#[command]
#[help_available]
#[aliases("rtc")]
#[only_in("guild")]
#[description("Restart TC")]
#[checks("ADMIN")]
pub async fn restart_tc(ctx: &Context, msg: &Message) -> CommandResult {
    if let Err(err) = msg.reply(&ctx.http, "Restarting TC").await {
        println!("Couldnt send reply message => {}", err)
    }

    cmd::new("/bin/systemctl")
        .arg("restart")
        .arg("tc")
        .spawn()
        .expect("Failed on restating TC");

    Ok(())
}

#[command]
#[help_available]
#[aliases("rtc2")]
#[only_in("guild")]
#[description("Restart TC2")]
#[checks("ADMIN")]
pub async fn restart_tc2(ctx: &Context, msg: &Message) -> CommandResult {
    if let Err(err) = msg.reply(&ctx.http, "Updating & Restarting TC2").await {
        println!("Couldnt send reply message => {}", err)
    }

    cmd::new("/bin/systemctl")
        .arg("restart")
        .arg("tc2")
        .spawn()
        .expect("Failed on systemctl cmd");

    Ok(())
}


// TODO: Move into its own folder (when other places use the same check)
#[check]
#[name = "ADMIN"]
async fn admin_check(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    let config_lock = {
        let data_read = ctx.data.read().await;

        // Only thing cloned here is the reference
        data_read
            .get::<Config>()
            .expect("Expecting config in client data")
            .clone()
    };

    let discord_config: Discord;

    {
        let config = config_lock.read().await;

        discord_config = config.discord.clone();
    }

    if let Some(admin_roles) = discord_config.admin_roles {
        for a_role in admin_roles {
            if msg
                .author
                .has_role(ctx, msg.guild_id.unwrap(), a_role)
                .await
                .unwrap()
            {
                return Ok(());
            }
        }
    }

    if let Some(owners_list) = discord_config.owners {
        for owner in owners_list {
            if msg.author.id == owner {
                return Ok(());
            }
        }
    }

    Err(Reason::Unknown)
}
