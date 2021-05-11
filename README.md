Sympho

===

simple discord music bot for pse.

## build

clone, set token to `DISCORD_TOKEN` env var, then `cargo build`

## Usage
```
!join: Join the VC channel with the user who called !join
!leave: leave from the current channel
!play <youtube url>: play music. supported single video, and playlist
!current: shows the title of the music currently playing
!volume <1 - 100>: set the music volume
!skip: skip the music currently playing
!pause: pause the music currently playing
!resume: resume the music currently playing
!stop: stop to the music currently playing(if there) and queue will be empty
```

## TODO

- [ ]: Songbirdの例から派生したので、コードを綺麗にする
- [ ]: `builtin-queue`に頼らず、`Sympho`自身の`queue`を持つコードに変更</br>
現状`Sympho`はソースを一つのみしか再生しないので、`builtin-queue`はここでは適さないため。
ssを再生する機能は時間があったら作るが、順番に再生ではなくミキシングにしないといけないので、結局`builtin-queue`は要らない。
`Track`は`skip`される可能性があるので、例えばPlaylistを投げた時にすべての動画に対して`Track`オブジェクトを作るのはナンセンスな気がする。
なので、`Sympho`では`Track`の`play`、 `skip`、と再生の終わりの後の次の曲がある時、に`queue`に入れるので、その時にのみ`Track`オブジェクトが作成される。
`builtin-queue`を使った理由はこれだけの機能でBotコードが700行くらいになるとは思わず、`Sympho`自身の`queue`を自分で作るのが面倒だったため。
