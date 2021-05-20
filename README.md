Sympho
===

simple discord music bot for my hobby.</br>
sympho equipped with a unique queue system.</br>

## build

clone, set token to `DISCORD_TOKEN` env var, then `cargo build`

## Usage
if want to call bot, need role named "DJUser".</br>
if want to get rid of this, look at the source code and remove the code call to `has_dj_user`.</br>

```markdown
# made by uru(ururu#5687)

- !play <youtube, soundcloud url> : play music. supported single video, and playlist
- !loop <on or off> : enable/disable loop the current playing song
- !volume <0.1 - 100.0> : set the music volume
- !queue <0 - ?> : Shows a list of songs in the queue. index is 0 first.

- !join : Join the VC channel with the user who called !join
- !leave : leave from the current channel
- !current(or nowplaying) : shows the title of the music currently playing
- !skip : skip the music currently playing
- !pause : pause the music currently playing
- !resume : resume the music currently playing
- !stop : stop to the music currently playing(if there) and queue will be empty
```

## TODO

-[x]: ファイル分けたので綺麗になった(主観) ~~Songbirdの例から派生したので、コードを綺麗にする~~

-[x]: `unwrap()`は`Sympho`から消滅したぜ... ~~`unwrap`は邪悪なので、コードから消し飛ばす。(今のところバレてないけど、`Sympho`をクラッシュさせるコマンドのやり方が存在してしまっているので)~~

-[x]: 対応した。 ~~`builtin-queue`に頼らず、`Sympho`自身の`queue`を持つコードに変更。
現状`Sympho`はソースを一つのみしか再生しないので、`builtin-queue`はここでは適さないため。
ssを再生する機能は時間があったら作るが、順番に再生ではなくミキシングにしないといけないので、結局`builtin-queue`は要らない。
`Track`は`skip`される可能性があるので、例えばPlaylistを投げた時にすべての動画に対して`Track`オブジェクトを作るのはナンセンスな気がする。
なので、`Sympho`では`Track`の`play`、 `skip`、と再生の終わりの後の次の曲がある時、に`queue`に入れるので、その時にのみ`Track`オブジェクトが作成される。
`builtin-queue`を使った理由はこれだけの機能でBotコードが700行くらいになるとは思わず、`Sympho`自身の`queue`を自分で作るのが面倒だったため。~~
