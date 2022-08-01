use std::collections::HashSet;

use serenity::framework::standard::{
    help_commands, macros::help, Args, CommandGroup, CommandResult, HelpOptions,
};

use serenity::model::{channel::Message, id::UserId};
use serenity::prelude::Context;

#[help] // Helpコマンド
#[individual_command_tip = "これはヘルプコマンド"] // Helpコマンドの説明
#[strikethrough_commands_tip_in_guild = ""] // 使用できないコマンドについての説明を削除
async fn my_help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;
    Ok(())
}
