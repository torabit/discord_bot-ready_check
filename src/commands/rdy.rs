use serenity::builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::futures::StreamExt;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::time::Duration;

#[derive(Debug, Copy, Clone, PartialEq)]
enum ReadyState {
    NotReady,
    Ready,
    TimedOut,
}

#[command]
#[description = "READY CHECK"]
async fn rdy(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    println!("Ready Check start.");

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let role_id = args.single::<String>().unwrap_or("".to_string());
    let limit = args.single::<u64>().unwrap_or(1);

    let mut members: Vec<Option<Member>> = Vec::new();

    // もし60分以上の指定があればエラー文
    if limit > 60 {
        let _ = &msg
            .channel_id
            .say(
                &ctx.http,
                format!("Specified time is too large!\nPlease enter less than 60 minutes"),
            )
            .await;
        return Ok(());
    }

    if role_id.len() == 0 {
        members = get_members_by_channel_id(&guild, channel_id);
    } else {
        let mut role_id_string = "".to_string();
        for (i, c) in role_id.chars().enumerate() {
            if i >= 3 && i < role_id.len() - 1 {
                role_id_string.push(c);
            }
        }
        match role_id_string.parse::<u64>() {
            Ok(role_id) => {
                members = get_members_by_role_id(&guild, RoleId(role_id));
            }
            Err(err) => {
                let _ = &msg
                    .channel_id
                    .say(&ctx.http, format!("No such a role name.\n{}", err))
                    .await;
                println!("Exceptionally terminated.");
                return Ok(());
            }
        }
    }

    // メンバーの配列が空の場合処理を終わらせる
    if members.is_empty() {
        let _ = &msg.channel_id.say(&ctx.http, format!("No members.")).await;
        println!("Exceptionally terminated.");
        return Ok(());
    }

    let mut ready_state_operation = MembersReadyStateOperation::new(members);
    let target_member = ready_state_operation.target_member_repl();

    // このメッセージのリアクションからready checkの判定をかける
    let start_message = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.set_embed(msg.create_start_embed(target_member, limit));
            m.reactions(vec![
                ReactionType::from('✅'),
                ReactionType::from('❌'),
            ])
        })
        .await?;
        
    let target_member_user_ids = ready_state_operation.target_member_user_ids();
    let mut collector = start_message
        .await_reactions(&ctx)
        .timeout(Duration::from_secs(60 * limit))
        .filter(|r| r.emoji.unicode_eq("✅") || r.emoji.unicode_eq("❌"))
        .await;

    while let Some(r) = collector.next().await {
        let react = r.as_inner_ref();
        match react.user_id {
            Some(user_id) => {
                if target_member_user_ids.contains(&user_id) {
                    if react.emoji.unicode_eq("✅") {
                        ready_state_operation.update_ready_state(user_id, ReadyState::Ready);
                    }
                    if react.emoji.unicode_eq("❌") {
                        ready_state_operation.update_ready_state(user_id, ReadyState::NotReady);
                    }
                };
            }
            None => {}
        }
        if ready_state_operation.answered_all() {
            break;
        }
    }

    let member_list = MemberList {
        ready_member: ready_state_operation.members_ready_state_repl(ReadyState::Ready),
        not_ready_member: ready_state_operation.members_ready_state_repl(ReadyState::NotReady),
        timed_out_member: ready_state_operation.members_ready_state_repl(ReadyState::TimedOut),
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

    // start message削除
    let _ = start_message.delete(&ctx.http).await;

    println!("Ready Check complete.");
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]

struct MembersReadyState {
    is_ready: ReadyState,
    user_id: UserId,
    user_name: String,
}
// 引数にIDと名前だけ取るようにする
impl MembersReadyState {
    fn new(member: &Option<Member>) -> Self {
        let user_id: UserId = {
            match member {
                Some(member) => member.user.id.to_owned(),
                None => UserId(0),
            }
        };
        let user_name: String = {
            match member {
                Some(member) => member.user.name.to_owned(),
                None => "名無し".to_string(),
            }
        };

        Self {
            is_ready: ReadyState::TimedOut,
            user_id,
            user_name,
        }
    }
}

#[derive(Debug)]

struct MembersReadyStateOperation {
    ready_states: Vec<MembersReadyState>,
}

impl MembersReadyStateOperation {
    fn new(members: Vec<Option<Member>>) -> Self {
        let ready_states = members
            .iter()
            .map(|x| MembersReadyState::new(x))
            .collect::<Vec<MembersReadyState>>();

        Self { ready_states }
    }

    fn target_member_user_ids(&self) -> Vec<UserId> {
        let user_ids = &self
            .ready_states
            .to_owned()
            .iter()
            .map(|x| x.user_id)
            .collect::<Vec<UserId>>()
            .to_vec();
        user_ids.to_vec()
    }

    fn target_member_repl(&self) -> String {
        let mut repl = String::new();

        repl.push_str(
            &self
                .ready_states
                .to_owned()
                .into_iter()
                .map(|x| x.user_name)
                .collect::<Vec<String>>()
                .join("\n"),
        );

        repl
    }

    fn members_ready_state_repl(&self, ready_state: ReadyState) -> String {
        let mut repl = String::new();

        repl.push_str(
            &self
                .ready_states
                .to_owned()
                .into_iter()
                .filter(|x| x.is_ready == ready_state)
                .map(|x| x.user_name)
                .collect::<Vec<String>>()
                .join("\n"),
        );

        repl
    }

    fn is_everyone_ready(&mut self) -> bool {
        let is_everyone_ready: bool = self
            .ready_states
            .iter()
            .all(|x| x.is_ready == ReadyState::Ready);
        is_everyone_ready
    }

    fn update_ready_state(&mut self, user_id: UserId, is_ready: ReadyState) {
        for i in 0..self.ready_states.len() {
            if self.ready_states[i].user_id == user_id {
                self.ready_states[i].is_ready = is_ready;
            }
        }
    }

    // is_readyのデフォルト値 ReadyState::TimedOut ではない場合回答したと判定
    fn answered_all(&mut self) -> bool {
        let mut answered = true;
        self.ready_states.iter().for_each(|x| {
            if let ReadyState::TimedOut = x.is_ready {
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

        create_footer_embed.text("⏳Ready Check @");
        create_footer_embed
    }
}

pub struct MemberList {
    ready_member: String,
    not_ready_member: String,
    timed_out_member: String,
}

pub trait MessageExt {
    fn create_start_embed(&self, member: String, limit: u64) -> CreateEmbed;
    fn create_successed_embed(&self, member: String) -> CreateEmbed;
    fn create_failure_embed(&self, member_list: MemberList) -> CreateEmbed;
}

impl MessageExt for Message {
    fn create_start_embed(&self, member: String, limit: u64) -> CreateEmbed {
        let mut start_embed = CreateEmbed::default();
        let author = &self.author;

        // 現在時間を取得して、引数のlimitの時間を足した時刻を表示させたい
        start_embed
            .colour(0x3498DB)
            .set_author(author.create_author())
            .title(format!("{} requested a Ready Check.", author.name))
            .field("Target Member", format!("{}", member), false)
            .description(format!("You have {} miniutes to react.", limit))
            .set_footer(author.create_footer())
            .timestamp(&self.timestamp);

        start_embed
    }

    fn create_successed_embed(&self, member: String) -> CreateEmbed {
        let mut successed_embed = CreateEmbed::default();
        let author = &self.author;

        successed_embed
            .colour(0x2ECC71)
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

fn get_members_by_channel_id(guild: &Guild, channel_id: Option<ChannelId>) -> Vec<Option<Member>> {
    let members = guild
        .voice_states
        .iter()
        .filter(|voice_state| voice_state.1.channel_id == channel_id)
        .map(|voice_state| voice_state.1.member.to_owned())
        .collect::<Vec<Option<Member>>>();

    members
}

fn get_members_by_role_id(guild: &Guild, role_id: RoleId) -> Vec<Option<Member>> {
    let mut members = Vec::new();

    for (_, member) in &guild.members {
        if member.roles.contains(&role_id) {
            members.push(Some(member.to_owned()));
        };
    }

    members
}
