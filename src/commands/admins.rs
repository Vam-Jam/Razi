use crate::RAZI_CONFIG;
use serenity::{
    framework::standard::{
        macros::{check, command},
        Args, CheckResult, CommandOptions, CommandResult,
    },
    model::channel::Message,
    prelude::Context,
};
use std::process::Command;

// This is just a command so i can restart the server when needed (and people find it funny every now and then)
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

    Command::new("shutdown")
        .arg("now")
        .spawn()
        .expect("Luckily died on shutdown");

    Ok(())
}


#[command]
#[help_available]
#[aliases("r_tc", "rtc")]
#[only_in("guild")]
#[description("Restart TC")]
#[checks("ADMIN")]
pub async fn restart_tc(ctx: &Context, msg: &Message) -> CommandResult {
	match msg
        .reply(
            &ctx.http,
            "Restarting TC",
        )
        .await
    {
        Err(err) => println!("Couldnt send reply message => {}", err),
        _ => (),
	}

	Command::new("systemctl")
        .arg("restart tc")
        .spawn()
        .expect("Failed on restating TC");
	
	Ok(())
}

#[command]
#[help_available]
#[aliases("r_ww", "rww")]
#[only_in("guild")]
#[description("Restart WW")]
#[checks("ADMIN")]
pub async fn restart_ww(ctx: &Context, msg: &Message) -> CommandResult {
	match msg
        .reply(
            &ctx.http,
            "Restarting WW",
        )
        .await
    {
        Err(err) => println!("Couldnt send reply message => {}", err),
        _ => (),
	}

	Command::new("systemctl")
        .arg("restart ww")
        .spawn()
        .expect("Failed on restating WW");
	
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
