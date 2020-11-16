use crate::settings::toml::*;
use crate::RAZI_CONFIG;
use serenity::{
    framework::standard::{
        macros::{check, command},
        Args, CheckResult, CommandOptions, CommandResult,
    },
    model::channel::Message,
    prelude::Context,
};
use std::env::current_dir;
use std::process::Command;

#[command]
#[help_available]
#[aliases("emergency_do_not_use")]
#[only_in("guild")]
#[description("This is a command you are only suppose to use in an emergency.\nLike when razi start's to come alive or something dumb.\n////DO NOT USE THIS IF THIS IS NOT THE CASE////")]
#[checks("ADMIN")]
pub async fn emergency(ctx: &Context, msg: &Message) -> CommandResult {
    match msg
        .reply(
            &ctx.http,
            "Well, you did the command you probably shouldn't have done. Goodbye",
        )
        .await
    {
        Err(err) => {
            println!("Couldnt send reply message => {}", err);
            return Ok(()); // return because they didnt get to see the message >:(
        }
        _ => (),
    }

    // TODO: Uninstall Razi

    Command::new("cargo")
        .current_dir(current_dir().unwrap())
        .arg("clean")
        .spawn()
        .expect("Luckily died on cargo clean");

    Command::new("shutdown")
        .arg("now")
        .spawn()
        .expect("Luckily died on shutdown");
        
    Ok(())
}

#[check]
#[name = "ADMIN"]
async fn admin_check(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> CheckResult {
    let config = RAZI_CONFIG.with(|cell| cell.borrow().to_owned());

    for admin_list in config.discord.admin_roles {
        match msg
            .author
            .has_role(ctx, msg.guild_id.unwrap(), admin_list)
            .await
        {
            Ok(result) => {
                if result {
                    return true.into();
                }
            }
            _ => (),
        };
    }

    false.into()
}
