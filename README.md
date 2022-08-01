# discord_bot-ready_check
Rust製のDiscord Bot、serenityを利用してます。

Discordの機能であるreactionを利用してVoiceチャンネル内のユーザーがReadyかどうか判定するBotです。
```zsh
git clone git@github.com:torabit/discord_bot-ready_check.git
cd discord_bot-ready_check.git
// root dirにconfig.jsonを作成してください {"token": "DISCORD_BOT_TOKEN"}
cargo run --relase
```
