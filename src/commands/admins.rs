use crate::RAZI_CONFIG;
use serenity::{
    framework::standard::{
        macros::{check, command},
        Args, CommandOptions, CommandResult, Reason,
    },
    model::channel::Message,
    prelude::Context,
};

use std::io::Write;
use std::process::{Command, Stdio};

// This is just a command so i can restart the server when needed (and people find it funny every now and then)
#[command]
#[help_available]
#[aliases("emergency_do_not_use")]
#[only_in("guild")]
#[description("This is a command you are only suppose to use in an emergency.\nLike when razi start's to come alive or something dumb.\n////DO NOT USE THIS IF THIS IS NOT THE CASE////")]
#[checks("ADMIN")]
pub async fn emergency(ctx: &Context, msg: &Message) -> CommandResult {
    if let Err(err) = msg
        .reply(
            &ctx.http,
            "Well, you did the command you probably shouldn't have done. Goodbye",
        )
        .await
    {
        println!("Could not send reply message => {}", err);
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
    if let Err(err) = msg.reply(&ctx.http, "Restarting TC").await {
        println!("Couldnt send reply message => {}", err)
    }

    Command::new("/bin/systemctl")
        .arg("restart")
        .arg("tc")
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
    if let Err(err) = msg.reply(&ctx.http, "Restarting WW").await {
        println!("Couldnt send reply message => {}", err)
    }

    Command::new("/bin/systemctl")
        .arg("restart")
        .arg("ww")
        .spawn()
        .expect("Failed on restating WW");

    Ok(())
}

#[command]
#[help_available]
#[aliases("r_mbu", "rmbu")]
#[only_in("guild")]
#[description("Restart MBU")]
#[checks("ADMIN")]
pub async fn restart_mbu(ctx: &Context, msg: &Message) -> CommandResult {
    if let Err(err) = msg.reply(&ctx.http, "Restarting MBU").await {
        println!("Couldnt send reply message => {}", err)
    }

    Command::new("/bin/systemctl")
        .arg("restart")
        .arg("mbu")
        .spawn()
        .expect("Failed on restating MBU");

    Ok(())
}

#[command]
#[help_available]
#[aliases("u_vs", "uvs")]
#[only_in("guild")]
#[description("Force update vintage story")]
#[checks("ADMIN")]
pub async fn update_vintage(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let config = RAZI_CONFIG.with(|cell| cell.borrow().to_owned());

    if args.is_empty() || config.vintage_server.is_none() {
        return Ok(());
    }

    // We only want the ending of the url
    let mut url_path = args.single::<String>().unwrap();

    // lol, maybe lets improve that later
    if url_path.contains("http") || url_path.contains("::") {
        if let Err(err) = msg
            .reply(&ctx.http, "Please do not send any custom urls!")
            .await
        {
            println!("Couldnt send reply message => {}", err)
        }

        return Ok(());
    }

    if url_path.starts_with('|') {
        url_path = url_path
            .chars()
            .next()
            .map(|c| &url_path[c.len_utf8()..])
            .unwrap()
            .to_string();
    }

    let wget_path = format!("wget https://cdn.vintagestory.at/gamefiles/{}", url_path);
    let bash_file = format!("bash {}", config.vintage_server.unwrap().bash_filename);

    // TODO: Clean up, extremely ugly/hacky, such is late night coding with lack of sleep
    let mut cmd = Command::new("/bin/su")
        .arg("-")
        .arg("jenny")
        .stdin(Stdio::piped())
        .spawn()
        .expect("Could not su as Jenny");

    {
        let stdin = cmd.stdin.as_mut().expect("Failed to get stdin");
        stdin.write_all(b"tmux attach").expect("Failed to write");
        stdin
            .write_all(b"AUTOMATED SHUTDOWN, UPDATING!")
            .expect("Failed to write");
        stdin.write_all(b"/stop").expect("Failed to write");
        stdin
            .write_all(wget_path.as_bytes())
            .expect("Failed to write");
        stdin
            .write_all(b"tar xzf vs_server_*.*.*.tar.gz")
            .expect("Failed to write");
        stdin
            .write_all(b"rm vs_server_*.*.*.tar.gz")
            .expect("Failed to write");
        stdin
            .write_all(bash_file.as_bytes())
            .expect("Failed to write");
    }

    cmd.kill().expect("Could not kill command");

    if let Err(err) = msg.reply(&ctx.http, "Auto update complete").await {
        println!("Couldnt send reply message => {}", err)
    }

    Ok(())
}

#[check]
#[name = "ADMIN"]
async fn admin_check(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    let config = RAZI_CONFIG.with(|cell| cell.borrow().to_owned());

    for admin_list in config.discord.admin_roles {
        if msg
            .author
            .has_role(ctx, msg.guild_id.unwrap(), admin_list)
            .await
            .is_ok()
        {
            return Ok(());
        }
    }

    Err(Reason::Unknown)
}
