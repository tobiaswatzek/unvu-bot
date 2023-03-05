mod commands;

use std::env;

use dotenv::dotenv;
use serenity::model::prelude::interaction::{Interaction, InteractionResponseType};
use serenity::model::prelude::{GuildId, Ready};
use serenity::prelude::{Context, EventHandler, GatewayIntents};
use serenity::{async_trait, Client};
use tracing::{debug, error, instrument, warn};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    #[instrument(skip(self, ctx))]
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            debug!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                "ping" => commands::ping::run(&command.data.options),
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                warn!("cannot respond to slash command: {}", why);
            }
        }
    }

    #[instrument(skip(self, ctx, ready))]
    async fn ready(&self, ctx: Context, ready: Ready) {
        debug!("bot {} is connected", ready.user.name);

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands.create_application_command(|command| commands::ping::register(command))
        })
        .await;

        if let Err(err) = commands {
            error!("error when setting commands for guild {}: {}", guild_id, err);
            panic!("guild commands could not be set");
        }

        debug!("set commands for guild {}", guild_id);
    }
}

#[tokio::main]
#[instrument]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    let secret = load_env().expect("error loading required environment variables");

    // Build our client.
    let mut client = Client::builder(secret, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("error creating client");

    if let Err(why) = client.start().await {
        error!("client error {}", why);
    }
}

fn load_env() -> Option<String> {
    const ENV_KEY: &str = "UNVU_BOT_SECRET";
    let secret = env::var(ENV_KEY).expect(format!("{ENV_KEY} has to be set").as_str());
    if secret.trim().is_empty() {
        error!("environment variable {} is empty or whitespace", ENV_KEY);
        return None;
    }
    Some(secret)
}
