use std::sync::Arc;

use serenity::{
    builder::CreateApplicationCommand,
    http::CacheHttp,
    model::prelude::{
        interaction::application_command::ApplicationCommandInteraction, ChannelType, Member,
    },
    prelude::Context,
};

use anyhow::{anyhow, Result};
use songbird::Songbird;
use tracing::{error, info};

pub async fn run(command: &ApplicationCommandInteraction, ctx: &Context) -> String {
    let manager = songbird::get(ctx)
        .await
        .expect("songbird voice client added at startup")
        .clone();
    let member = command
        .member
        .as_ref()
        .expect("member was expected on the command");

    join_channel_of_member(member, manager, &ctx)
        .await
        .expect("join to voice was not successful");
    return "Bing!".to_string();

    // if let Some(handler_lock) = manager.get(command.guild_id.unwrap()) {
    //     let mut handler = handler_lock.lock().await;

    //     let sources_lock = ctx
    //         .data
    //         .read()
    //         .await
    //         .get::<SoundStore>()
    //         .cloned()
    //         .expect("Sound cache was installed at startup.");
    //     let sources = sources_lock.lock().await;
    //     let source = sources
    //         .get("ting")
    //         .expect("Handle placed into cache at startup.");

    //     let _sound = handler.play_source(source.into());

    //     return "Bing!".to_string();
    // } else {
    //     return "Please join a voice channel first".to_string();
    // }

    // "Bing!".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("bing")
        .description("This plays a little bing sound.")
}

async fn join_channel_of_member(
    member: &Member,
    manager: Arc<Songbird>,
    ctx: &Context,
) -> Result<()> {
    // TOOD: handle errors more graceful
    let channels = member.guild_id.channels(ctx.http.clone()).await?;

    let mut voice_channel = None;
    for channel in channels.values().filter(|c| c.kind == ChannelType::Voice) {
        let members = channel.members(ctx.cache.clone()).await?;
        if members.iter().any(|m| m.user.id == member.user.id) {
            voice_channel = Some(channel);
            break;
        }
    }

    if voice_channel.is_none() {
        return Err(anyhow!("no voice channel could be found"));
    }

    let channel_id = voice_channel.unwrap().id;

    let (handler_lock, success_reader) = manager.join(member.guild_id, channel_id).await;

    //let call_lock_for_evt = Arc::downgrade(&handler_lock);

    if let Ok(_reader) = success_reader {
        //  let mut handler = handler_lock.lock().await;
        info!("joined channel {}", channel_id);
    } else {
        error!("could not join channel {}", channel_id);
    }

    Ok(())
}
