Sympho
===

simple discord music bot for my hobby.</br>

## Feature

- written by Rust✨
- play videos(you know, audio only) from many sites(depends on youtube-dl extractors)
- play all from playlist
- no using songbird's builtin-queue, sympho have unique queue system
- Restrictions on command use based on role name(on default, sympho will check user have role that name called `DJUser`). you can remove it from the code if dont need it
- enough commands(default prefix is `!`, u can change define env `SYMPHO_PREFIX`)</br>
  <details>
    <summary>command list (click to expand/collapse)</summary>

    - `help Option<command name>` :</br>Show the command list, or if set command name on arg, show the command description.

    - `join` :</br>Join the VC channel with the user who called join command.(and if bot not playing the music on other channel)

    - `leave` :</br>Leave from the current channel.

    - `play <url>, play with file upload` :</br>Start to play music. supported some site, support playlist, and file upload

    - `stop` :</br>Stop to the music currently playing(if there) and queue will be empty.

    - `volume` :</br>Set the music volume. range is 0.0 ~ 100.0.

    - `pause` :</br>Pause the music currently playing.

    - `resume` :</br>Resume the music currently playing.

    - `skip` :</br>Skip the music currently playing.

    - `loop <on/off>` :</br>Enable/Disable loop the current playing song.

    - `current` :</br>Shows the info of the music currently playing.

    - `queue` :</br>Shows a list of songs in the queue. index is 0 first.
  </details>

## Advantages over other bots

- All written in Rust
- Completely open source, and Easy to build (just set `DISCORD_TOKEN` env then run `cargo run`)
- Sympho can play the uploaded audio file
- Depends on youtube-dl, but not limited to Youtube or Soundcloud URLs(For example, sympho also accept bilibili videos)
- There are no premier restrictions like Rythm bot etc.
- The code isn't too dirty, so anyone can customize it as like, for self-host

## Dis-Advantages over other bots

- Missing commands
- Self-hosting costs
- Difficult to customize for people who can't Rust
- There is no such thing as an effect function for song
- The audio area is still unstable(there is a problem with the sound being played, or the sound at the start of playback is a little strange.)

## Require

- rust
- ffmpeg
- libopus
- youtube-dl

see more information [Songbird's README#dependencies](https://github.com/serenity-rs/songbird#dependencies)

tested on Windows10 x64 and Ubuntu 20.04 with rust nightly 1.53 ~ 1.54

## Build and run

clone, set token to `DISCORD_TOKEN` env var, then `cargo build`, then run it.

## TODO

- [ ] add more command(e.g. queue manage, etc.)
- [x] refactoring
- [x] remove all `unwrap()`
- [x] make unique queue system, not `builtin-queue`
