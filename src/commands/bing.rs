use std::sync::Arc;

use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        ChannelType, Member,
    },
    prelude::{Context, Mutex},
};

use anyhow::{anyhow, Result};
use songbird::{
    input::{self, cached::Memory, Input},
    Call, Songbird,
};
use tokio::time::sleep;
use tracing::{error, info, instrument};

#[instrument(skip(command, ctx))]
pub async fn run(command: &ApplicationCommandInteraction, ctx: &Context) {
    match run_internal(&command, &ctx).await {
        Ok(_) => {}
        Err(err) => {
            answer_with_error_message(
                "An internal error occurred while processing your command.",
                &command,
                &ctx,
            )
            .await;
            error!("An error occurred while handling the command {}.", err);
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("bing")
        .description("This plays a little bing sound.")
}

async fn run_internal(command: &ApplicationCommandInteraction, ctx: &Context) -> Result<()> {
    let manager = songbird::get(ctx).await.map(|m| m.clone()).ok_or(anyhow!(
        "Could not load sound manager which should be added on startup."
    ))?;

    let member = command
        .member
        .as_ref()
        .ok_or(anyhow!("Could not get member from command."))?;

    let call_lock = join_channel_of_member(member, manager, &ctx).await?;

    answer_playing(command, ctx).await;
    play_sound(call_lock.clone()).await?;

    leave_channel(call_lock.clone()).await?;

    Ok(())
}

async fn answer_playing(command: &ApplicationCommandInteraction, ctx: &Context) {
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message
                        .content(format!("😊 Playing your sound"))
                        .ephemeral(true)
                })
        })
        .await
        .expect("could not answer with error message");
}

async fn answer_with_error_message(
    error_message: &str,
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) {
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message
                        .content(format!("😖 {error_message}"))
                        .ephemeral(true)
                })
        })
        .await
        .expect("could not answer with error message");
}

async fn play_sound(call_lock: Arc<Mutex<Call>>) -> Result<()> {
    let mut bing = Memory::new(input::ffmpeg("bing.wav").await?)?;
    bing.raw.load_all();
    let bing_src: Input = bing.new_handle().try_into()?;
    let duration = bing_src
        .metadata
        .duration
        .ok_or(anyhow!("Duration must be available"))?;

    let mut call = call_lock.lock().await;
    let t = call.play_only_source(bing_src);
    sleep(duration).await;

    Ok(())
}

async fn leave_channel(call_lock: Arc<Mutex<Call>>) -> Result<()> {
    call_lock.lock().await.leave().await?;
    Ok(())
}

async fn join_channel_of_member(
    member: &Member,
    manager: Arc<Songbird>,
    ctx: &Context,
) -> Result<Arc<Mutex<Call>>> {
    let channels = member.guild_id.channels(ctx.http.clone()).await?;
    let mut voice_channel = None;
    for channel in channels.values().filter(|c| c.kind == ChannelType::Voice) {
        let members = channel.members(ctx.cache.clone()).await?;
        if members.iter().any(|m| m.user.id == member.user.id) {
            voice_channel = Some(channel);
            break;
        }
    }

    let channel_id = voice_channel
        .ok_or(anyhow!("No voice channel could be found"))?
        .id;

    let (handler_lock, success_reader) = manager.join(member.guild_id, channel_id).await;

    if let Err(err) = success_reader {
        error!("could not join channel {}", channel_id);
        return Err(err.into());
    }

    info!("joined channel {}", channel_id);
    Ok(handler_lock)
}
