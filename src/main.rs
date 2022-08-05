mod commands;
mod utils;

use serenity::async_trait;
use serenity::client::{bridge::gateway::GatewayIntents, Client};
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::model::gateway::Ready;
use serenity::prelude::{Context, EventHandler};

use commands::{help::*, rdy::*};
use utils::token::Token;

// Handler構造体。取得したいイベントを実装する
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Botが起動したときに走る処理
    async fn ready(&self, _: Context, ready: Ready) {
        // デフォルトでC言語のprintfのようなことができる
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[description("汎用コマンド")]
#[summary("一般")]
#[commands(rdy)]

struct General;

#[tokio::main]
async fn main() {
    // Discord Bot Token を設定
    let token = Token::get_token("config.json").expect("Err トークンが見つかりません");
    // コマンド系の設定
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // コマンドプレフィックス
        .help(&MY_HELP) // ヘルプコマンドを追加
        .group(&GENERAL_GROUP); // general を追加するには,GENERAL_GROUP とグループ名をすべて大文字にする

    // Botのクライアントを作成
    let mut client = Client::builder(&token)
        .event_handler(Handler) // 取得するイベント
        .framework(framework) // コマンドを登録
        .intents(GatewayIntents::all())
        .await
        .expect("Err creating client"); // エラーハンドリング

    // メインループ。Botを起動
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
