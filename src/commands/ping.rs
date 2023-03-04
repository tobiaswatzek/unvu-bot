use serenity::{model::prelude::interaction::application_command::CommandDataOption, builder::CreateApplicationCommand};

pub fn run(_options: &[CommandDataOption]) -> String {
    "Wow, this is so rusty!".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("ping").description("This gives you a message back.")
}
