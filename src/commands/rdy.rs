use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::time::Instant;

const READY: char = '✅';
const NOT_READY: char = '❌';

#[command]
#[description = "READY CHECK"]
async fn rdy(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    // 現在のVCにいるユーザーたちのReady State
    let mut ready_states = guild
        .voice_states
        .iter()
        .filter(|voice_state| voice_state.1.channel_id == channel_id)
        .map(|voice_state| ReadyState::new(voice_state.1.to_owned()))
        .collect::<Vec<ReadyState>>();

    // embedに表示するメンバーを抽出
    let mut target_member = String::new();
    target_member.push_str(
        &ready_states
            .to_owned()
            .into_iter()
            .map(|x| x.user_name)
            .collect::<Vec<String>>()
            .join("\n"),
    );

    // embedを作成
    // このメッセージのリアクションからready checkの判定をかける
    let embed_message = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.colour(0x3498DB)
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
        if Instant::now().duration_since(start_time).as_secs() > 30 || answered_all(&ready_states) {
            break;
        }

        let ready_users = embed_message
            .reaction_users(&ctx.http, READY, Some(50u8), UserId(0))
            .await?
            .to_owned()
            .into_iter()
            .map(|x| x.id)
            .collect::<Vec<UserId>>();

        let not_ready_users = embed_message
            .reaction_users(&ctx.http, NOT_READY, Some(50u8), UserId(0))
            .await?
            .to_owned()
            .into_iter()
            .map(|x| x.id)
            .collect::<Vec<UserId>>();
        
        for i in 0..ready_states.len() {
            for j in 0..ready_users.len() {
                if ready_states[i].user_id == ready_users[j] {
                    ready_states[i].is_ready = Some(true);
                }
            }

            for k in 0..not_ready_users.len() {
                if ready_states[i].user_id == not_ready_users[k] {
                    ready_states[i].is_ready = Some(false);
                }
            }
        }
    }

    let is_ready: bool = ready_states.iter().all(|x| x.is_ready == Some(true));
    // embedに表示するメンバーを抽出

    let mut ready_member = String::new();
    let mut not_ready_member = String::new();

    ready_member.push_str(
      &ready_states
      .to_owned()
      .into_iter()
      .filter(|x| x.is_ready == Some(true))
      .map(|x| x.user_name)
      .collect::<Vec<String>>()
      .join("\n")
    );

    not_ready_member.push_str(
      &ready_states
      .to_owned()
      .into_iter()
      .filter(|x| x.is_ready == Some(false))
      .map(|x| x.user_name)
      .collect::<Vec<String>>()
      .join("\n")
    );

    if is_ready {
      msg
      .channel_id
      .send_message(&ctx.http, |m| {
          m.embed(|e| {
              e.colour(0x00ff00)
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
      msg
      .channel_id
      .send_message(&ctx.http, |m| {
          m.embed(|e| {
              e.colour(0xED4245)
                  .title("Ready Check complete.\nSomeone is Not Ready.")
                  .field("Ready Member", format!("{}", noone_response(ready_member)), false)
                  .field("Not Ready Member", format!("{}", noone_response(not_ready_member)), false)
                  .footer(|f| {
                      f.text("⏱ Ready Check @");
                      f
                  })
                  .timestamp(&msg.timestamp)
          })
      })
      .await?;
    }
    println!("end");
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

// is_readyのデフォルト値 None が書き換えられているか判定
fn answered_all(ready_states: &Vec<ReadyState>) -> bool {
    let mut answered = true;
    ready_states.iter().for_each(|x| {
        if let None = x.is_ready {
            answered = false;
        }
    });
    answered
}

fn noone_response(value: String) -> String {
  if value.len() < 1 {
    return "no one answered".to_string();
  } else {
    return value;
  }
}