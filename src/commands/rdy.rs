use serenity::builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter};
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::time::Instant;

const READY: char = '✅';
const NOT_READY: char = '❌';

#[command]
#[description = "READY CHECK"]
async fn rdy(ctx: &Context, msg: &Message) -> CommandResult {
    println!("Ready Check start.");

    let guild = msg.guild(&ctx.cache).await.unwrap();

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let mut ready_state_operation = ReadyStateOperation::new(guild, channel_id);
    let target_member = ready_state_operation.get_target_member();

    // embedを作成
    // このメッセージのリアクションからready checkの判定をかける
    let embed_message = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.colour(0x3498DB)
                    .set_author(msg.author.create_author())
                    .title(format!("{} requested a Ready Check.", msg.author.name))
                    .field("Target Member", format!("{}", target_member), false)
                    .description("You have to 30 seconds to answer.")
                    .footer(|f| {
                        f.text("⏱ Ready Check @");
                        f
                    })
                    .timestamp(&msg.timestamp)
            })
        })
        .await?;

    //　reactの初期化をする
    embed_message.react(&ctx.http, READY).await?;
    embed_message.react(&ctx.http, NOT_READY).await?;
    let start_time = Instant::now();
    // 30秒待つ
    // 時間計測で強制的にloopさせてるけど、もっといいやり方あるかも
    loop {
        let answered_all = ready_state_operation.answered_all();

        if Instant::now().duration_since(start_time).as_secs() > 30 || answered_all {
            break;
        }

        embed_message
            .reaction_users(&ctx.http, READY, Some(50u8), UserId(0))
            .await?
            .to_owned()
            .into_iter()
            .for_each(|x| ready_state_operation.update_is_ready(x.id, Some(true)));

        embed_message
            .reaction_users(&ctx.http, NOT_READY, Some(50u8), UserId(0))
            .await?
            .to_owned()
            .into_iter()
            .for_each(|x| ready_state_operation.update_is_ready(x.id, Some(false)));
    }

    let ready_member = ready_state_operation.ready_member_repl();
    let not_ready_member = ready_state_operation.not_ready_member_repl();
    let timeout_member = ready_state_operation.timeout_member_repl();
    let is_everyone_ready = ready_state_operation.is_everyone_ready();

    if is_everyone_ready {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.colour(0x2ecc71)
                        .title("Ready Check complete.\nAll player are Ready.")
                        .field("Ready Member", format!("{}", ready_member), false)
                        .footer(|f| {
                            f.text("⏱ Ready Check @");
                            f
                        })
                        .timestamp(&msg.timestamp)
                })
            })
            .await?;
    } else {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.colour(0xED4245)
                        .title("Ready Check complete.\nSomeone is Not Ready or Timeout.")
                        .field(
                            "Ready Member",
                            format!("{}", noone_response(ready_member)),
                            false,
                        )
                        .field(
                            "Not Ready Member",
                            format!("{}", noone_response(not_ready_member)),
                            false,
                        )
                        .field(
                            "Timeout Member",
                            format!("{}", noone_response(timeout_member)),
                            false,
                        )
                        .footer(|f| {
                            f.text("⏱ Ready Check @");
                            f
                        })
                        .timestamp(&msg.timestamp)
                })
            })
            .await?;
    }
    println!("Ready Check complete.");
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
struct ReadyState {
    is_ready: Option<bool>,
    user_id: UserId,
    user_name: String,
}

impl ReadyState {
    fn new(voice_state: VoiceState) -> Self {
        let user_id = voice_state.user_id;
        let user_name: String = {
            match voice_state.member {
                Some(member) => member.user.name,
                None => "名無し".to_string(),
            }
        };

        Self {
            is_ready: None,
            user_id,
            user_name,
        }
    }
}

#[derive(Debug)]
struct ReadyStateOperation {
    ready_states: Vec<ReadyState>,
}

impl ReadyStateOperation {
    fn new(guild: Guild, channel_id: Option<ChannelId>) -> Self {
        let ready_states = guild
            .voice_states
            .iter()
            .filter(|voice_state| voice_state.1.channel_id == channel_id)
            .map(|voice_state| ReadyState::new(voice_state.1.to_owned()))
            .collect::<Vec<ReadyState>>();

        Self { ready_states }
    }

    fn get_target_member(&self) -> String {
        let mut target_member = String::new();

        target_member.push_str(
            &self
                .ready_states
                .to_owned()
                .into_iter()
                .map(|x| x.user_name)
                .collect::<Vec<String>>()
                .join("\n"),
        );

        target_member
    }

    fn ready_member_repl(&self) -> String {
        let mut ready_member = String::new();

        ready_member.push_str(
            &self
                .ready_states
                .to_owned()
                .into_iter()
                .filter(|x| x.is_ready == Some(true))
                .map(|x| x.user_name)
                .collect::<Vec<String>>()
                .join("\n"),
        );

        ready_member
    }

    fn not_ready_member_repl(&self) -> String {
        let mut not_ready_member = String::new();

        not_ready_member.push_str(
            &self
                .ready_states
                .to_owned()
                .into_iter()
                .filter(|x| x.is_ready == Some(false))
                .map(|x| x.user_name)
                .collect::<Vec<String>>()
                .join("\n"),
        );

        not_ready_member
    }

    fn timeout_member_repl(&self) -> String {
        let mut timeout_member = String::new();

        timeout_member.push_str(
            &self
                .ready_states
                .to_owned()
                .into_iter()
                .filter(|x| x.is_ready == None)
                .map(|x| x.user_name)
                .collect::<Vec<String>>()
                .join("\n"),
        );

        timeout_member
    }

    fn is_everyone_ready(&mut self) -> bool {
        let is_everyone_ready: bool = self.ready_states.iter().all(|x| x.is_ready == Some(true));
        is_everyone_ready
    }

    fn update_is_ready(&mut self, user_id: UserId, is_ready: Option<bool>) {
        for i in 0..self.ready_states.len() {
            if self.ready_states[i].user_id == user_id {
                self.ready_states[i].is_ready = is_ready;
            }
        }
    }

    // is_readyのデフォルト値 None ではない場合回答したと判定
    fn answered_all(&mut self) -> bool {
        let mut answered = true;
        self.ready_states.iter().for_each(|x| {
            if let None = x.is_ready {
                answered = false;
            }
        });
        answered
    }
}

fn noone_response(value: String) -> String {
    if value.len() < 1 {
        return "no one here".to_string();
    } else {
        return value;
    }
}

pub trait UserExt {
    fn create_author(&self) -> CreateEmbedAuthor;
    fn create_footer(&self) -> CreateEmbedFooter;
}

impl UserExt for User {
    fn create_author(&self) -> CreateEmbedAuthor {
        let mut create_author_embed = CreateEmbedAuthor::default();

        create_author_embed.name(&self.name);

        if let Some(avatar) = &self.avatar_url() {
            create_author_embed.icon_url(avatar);
        } else {
            create_author_embed.icon_url(&self.default_avatar_url());
        }

        create_author_embed
    }

    fn create_footer(&self) -> CreateEmbedFooter {
        let mut create_footer_embed = CreateEmbedFooter::default();

        create_footer_embed
            .text("Created by @tora_tora_bit")
            .icon_url("https://avatars.githubusercontent.com/u/82490317?v=4");

        create_footer_embed
    }
}

}
