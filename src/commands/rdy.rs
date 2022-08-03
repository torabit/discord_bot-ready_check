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
    // Loop処理をBreakするのに必要な開始時間
    let start_time = Instant::now();
    let guild = msg.guild(&ctx.cache).await.unwrap();

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let mut ready_state_operation = ReadyStateOperation::new(guild, channel_id);
    let target_member = ready_state_operation.get_target_member();
    // このメッセージのリアクションからready checkの判定をかける
    let start_message = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.set_embed(msg.create_start_embed(target_member))
        })
        .await?;
    //　reactの初期化をする
    start_message.react(&ctx.http, READY).await?;
    start_message.react(&ctx.http, NOT_READY).await?;

    // 30秒待つ
    // 時間計測で強制的にloopさせてるけど、もっといいやり方あるかも
    loop {
        let answered_all = ready_state_operation.answered_all();

        if Instant::now().duration_since(start_time).as_secs() > 30 || answered_all {
            break;
        }

        start_message
            .reaction_users(&ctx.http, READY, Some(50u8), UserId(0))
            .await?
            .into_iter()
            .for_each(|x| ready_state_operation.update_is_ready(x.id, Some(true)));

        start_message
            .reaction_users(&ctx.http, NOT_READY, Some(50u8), UserId(0))
            .await?
            .into_iter()
            .for_each(|x| ready_state_operation.update_is_ready(x.id, Some(false)));
    }

    let member_list = MemberList {
        ready_member: ready_state_operation.ready_member_repl(),
        not_ready_member: ready_state_operation.not_ready_member_repl(),
        timed_out_member: ready_state_operation.timed_out_member_repl(),
    };

    if ready_state_operation.is_everyone_ready() {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.set_embed(msg.create_successed_embed(member_list.ready_member))
            })
            .await?;
    } else {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.set_embed(msg.create_failure_embed(member_list))
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

    fn timed_out_member_repl(&self) -> String {
        let mut timed_out_member = String::new();

        timed_out_member.push_str(
            &self
                .ready_states
                .to_owned()
                .into_iter()
                .filter(|x| x.is_ready == None)
                .map(|x| x.user_name)
                .collect::<Vec<String>>()
                .join("\n"),
        );

        timed_out_member
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

pub struct MemberList {
    ready_member: String,
    not_ready_member: String,
    timed_out_member: String,
}

pub trait MessageExt {
    fn create_start_embed(&self, member: String) -> CreateEmbed;
    fn create_successed_embed(&self, member: String) -> CreateEmbed;
    fn create_failure_embed(&self, member_list: MemberList) -> CreateEmbed;
}

impl MessageExt for Message {
    fn create_start_embed(&self, member: String) -> CreateEmbed {
        let mut start_embed = CreateEmbed::default();
        let author = &self.author;

        start_embed
            .colour(0x3498DB)
            .set_author(author.create_author())
            .title(format!("{} requested a Ready Check.", author.name))
            .field("Target Member", format!("{}", member), false)
            .description("You have to 30 seconds to answer.")
            .set_footer(author.create_footer())
            .timestamp(&self.timestamp);

        start_embed
    }

    fn create_successed_embed(&self, member: String) -> CreateEmbed {
        let mut successed_embed = CreateEmbed::default();
        let author = &self.author;

        successed_embed
            .colour(0x3498DB)
            .set_author(author.create_author())
            .title("Ready Check complete.\nAll player are Ready.")
            .field("Ready Member", format!("{}", member), false)
            .set_footer(author.create_footer())
            .timestamp(&self.timestamp);

        successed_embed
    }

    fn create_failure_embed(&self, member_list: MemberList) -> CreateEmbed {
        let mut failure_embed = CreateEmbed::default();
        let author = &self.author;

        failure_embed
            .colour(0xED4245)
            .set_author(author.create_author())
            .title("Ready Check complete.\nSomeone is Not Ready or Timed out.")
            .set_footer(author.create_footer())
            .timestamp(&self.timestamp);

        if member_list.ready_member.len() > 1 {
            failure_embed.field(
                "Ready Member",
                format!("{}", member_list.ready_member),
                false,
            );
        }

        if member_list.not_ready_member.len() > 1 {
            failure_embed.field(
                "Not Ready Member",
                format!("{}", member_list.not_ready_member),
                false,
            );
        }

        if member_list.timed_out_member.len() > 1 {
            failure_embed.field(
                "Timed out Member",
                format!("{}", member_list.timed_out_member),
                false,
            );
        }

        failure_embed
    }
}
