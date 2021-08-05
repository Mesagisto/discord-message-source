use std::sync::Arc;

use serenity::{
    client::Context,
    framework::standard::{
        Args, CommandResult,
        macros::command
    },
    model::{channel::Message}
};

use crate::config::CONFIG;


#[command]
#[required_permissions("ADMINISTRATOR")]
#[description =
"Mesagisto commands about channel
有关信使频道的指令"
]
#[sub_commands(set)]
pub async fn channel(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(
            ctx,
            "Please use one of the subcommands! (set, remove, show)",
        )
        .await?;
    Ok(())
}

#[command]
#[required_permissions("ADMINISTRATOR")]
#[description =
"Set mesagisto-channel for current channel
为当前频道设置信使频道"
]
#[usage = "<channel>"]
#[min_args(1)]
pub async fn set(ctx: &Context, msg: &Message,mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.reply(ctx,"<channel> parameter is empty").await?;
    }
    let channel = args.single::<String>().unwrap();
    let dc_channel_id = msg.channel_id.as_u64().to_string();
    CONFIG.target_address_mapper.insert(Arc::new(dc_channel_id), Arc::new(channel));
    msg.reply(ctx, "Successfully set the mesagisto channel of the current channel").await?;
    Ok(())
}