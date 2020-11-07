use crate::settings::toml::*;
use crate::RAZI_CONFIG;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::Context,
};
use std::env::current_dir;
use std::process::Command;

#[command]
#[help_available]
#[aliases("emergency_do_not_use")]
#[description("This is a command you are only suppose to use in an emergency.\nDO NOT USE THIS IF THIS IS NOT THE CASE")]
#[allowed_roles("361262801387520000", "474289311206866944", "364040665149079564")]
pub async fn emergency(ctx: &Context, msg: &Message) -> CommandResult {
    let mut config = RaziConfig::new();

    RAZI_CONFIG.with(|cell| {
        config = cell.borrow().clone();
    });

    let mut is_admin = false;

    for admin_list in config.discord.admin_roles {
        match msg
            .author
            .has_role(ctx, msg.guild_id.unwrap(), admin_list)
            .await
        {
            Ok(result) => {
                if result {
                    is_admin = true;
                    break;
                }
            }
            Err(err) => println!("Error with has_role check => {}", err), // maybe return instead of checking other roles?
        };
    }

    if is_admin {
        match msg
            .reply(
                ctx,
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
    } else {
        match msg
            .reply(ctx, "Sorry but you don't have the perms to do that.")
            .await
        {
            Err(err) => println!("Couldnt send reply message => {}", err),
            _ => (),
        }
    }

    Ok(())
}
