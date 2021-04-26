use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::razi_toml::Config;

#[command]
#[owners_only]
async fn reload_config(ctx: &Context, msg: &Message) -> CommandResult {
    let config_lock = {
        let data_read = ctx.data.read().await;

        // Only thing cloned here is the reference
        data_read
            .get::<Config>()
            .expect("Expecting config in client data")
            .clone()
    };

    {
        let mut config = config_lock.write().await;

        config.load_config();
    }

    msg.reply(ctx, "Config has been reloaded!").await?;

    Ok(())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}
