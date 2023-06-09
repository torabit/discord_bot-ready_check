# discord_bot-ready_check
Rust製のDiscord Bot、serenityを利用してます。

# Usage
### Collecting votes
![image](https://github.com/torabit/discord_bot-ready_check/assets/82490317/00962d48-9996-4bd8-ab1a-ad1db88ce824)
### Result votes
![image](https://github.com/torabit/discord_bot-ready_check/assets/82490317/5941be8d-7681-4630-9407-d42d4949c7c8)


Discordの機能であるreactionを利用してVoiceチャンネル内のユーザーがReadyかどうか判定するBotです。
```zsh
# shell

git clone git@github.com:torabit/discord_bot-ready_check.git

cd discord_bot-ready_check.git

cargo build

DISCORD_TOKEN=<use your token here> ./target/debug/discord-help-bot
```
## Ready Checkとは
Ready Check Botはターゲットメンバーが準備できているかどうかを制限時間内に判定します。

ターゲットの回答に応じた結果がDiscordにembedとして出力されます。

## 使い方
VCチャンネルに接続している場合は以下のコマンドで同じVC内のメンバーをターゲットにReadyCheckを行います。

参加者全員が回答するか1分経過すると自動で締め切ります。
```command
# discord
~rdy
```

ある特定のroleを持つユーザーをターゲットにしたい場合は以下のコマンドで特定のroleを持つユーザをターゲットにReadyCheckを行います。

ロールの次に数字を入力することで制限時間を設定できます。（最大60分、デフォルトでは1分が設定されています。）
```command
# discord
~rdy @role 60
```
