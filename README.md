Sympho
===

simple discord music bot for my hobby.</br>

## Feature

- written by Rust✨
- play videos(you know, audio only) from many sites(depends on youtube-dl extractors)
- play all from playlist
- no using songbird's builtin-queue, sympho have unique queue system
- enough commands
    - `help Option<command name>` :</br>Show the command list, or if set command name on arg, show the command description.

    - `join` :</br>Join the VC channel with the user who called join command.(and if bot not playing the music on other channel)

    - `leave` :</br>Leave from the current channel.

    - `play <url>` :</br>Start to play music. supported some site, support playlist.

    - `stop` :</br>Stop to the music currently playing(if there) and queue will be empty.

    - `volume` :</br>Set the music volume. range is 0.0 ~ 100.0.

    - `pause` :</br>Pause the music currently playing.

    - `resume` :</br>Resume the music currently playing.

    - `skip` :</br>Skip the music currently playing.

    - `loop <on/off>` :</br>Enable/Disable loop the current playing song.\nusage: <PREFIX>loop on

    - `current` :</br>Shows the info of the music currently playing.

    - `queue` :</br>Shows a list of songs in the queue. index is 0 first.

## Require

- rust
- ffmpeg
- libopus
- youtube-dl

see more information [Songbird's README#dependencies](https://github.com/serenity-rs/songbird#dependencies)

tested on Windows10 x64 and Ubuntu 20.04 with rust nightly 1.53 ~ 1.54

## Build and run

clone, set token to `DISCORD_TOKEN` env var, then `cargo build`, then run it.

## ~~TODO~~(Done)

- [x] ファイル分けたので綺麗になった(主観) ~~Songbirdの例から派生したので、コードを綺麗にする~~

- [x] `unwrap()`は`Sympho`から消滅したぜ... ~~`unwrap`は邪悪なので、コードから消し飛ばす。(今のところバレてないけど、`Sympho`をクラッシュさせるコマンドのやり方が存在してしまっているので)~~

- [x] 対応した。 ~~`builtin-queue`に頼らず、`Sympho`自身の`queue`を持つコードに変更。
現状`Sympho`はソースを一つのみしか再生しないので、`builtin-queue`はここでは適さないため。
ssを再生する機能は時間があったら作るが、順番に再生ではなくミキシングにしないといけないので、結局`builtin-queue`は要らない。
`Track`は`skip`される可能性があるので、例えばPlaylistを投げた時にすべての動画に対して`Track`オブジェクトを作るのはナンセンスな気がする。
なので、`Sympho`では`Track`の`play`、 `skip`、と再生の終わりの後の次の曲がある時、に`queue`に入れるので、その時にのみ`Track`オブジェクトが作成される。
`builtin-queue`を使った理由はこれだけの機能でBotコードが700行くらいになるとは思わず、`Sympho`自身の`queue`を自分で作るのが面倒だったため。~~
