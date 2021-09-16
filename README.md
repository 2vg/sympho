Sympho
===

simple discord music bot for my hobby.</br>

## Still WIP
Sympho is still under development, and unstable.</br>
I think the current implementation can handle most use cases, but I'm aiming for a great Discord Music Bot made with Rust!</br>
I guess features change frequently, but i promise that the commands and build methods will remain almost unchanged.(maybe :3)</br>
See the [TODO](#todo) section for information on features to be implemented, etc.</br>

## Feature

- written by Rust ✨
- because ↑, low binary size, blazing fast, and very small memory footprint 🚀
- play videos(you know, audio only) from many sites(depends on youtube-dl extractors) 🎥
- support to play from playlist, it also can be shuffled 🎶
- no using songbird's builtin-queue, sympho have unique queue system 💪
- Restrictions on command use based on role name(on default, sympho will check user have role that name called `DJUser`). you can remove it from the code if dont need it (**recommanded to remove this if u want to share to use Bot**) 👷
- enough commands(default prefix is `!`, u can change define env `SYMPHO_PREFIX`) 📌</br>
  <details>
    <summary>command list (click to expand/collapse)</summary>

    - `help Option<command name>` :</br>Show the command list, or if set command name on arg, show the command description.

    - `join` :</br>Join the VC channel with the user who called join command.(and if bot not playing the music on other channel)

    - `leave` :</br>Leave from the current channel.

    - `play <url>, <some keywords>, play with file upload` :</br>
      Start to play music. supported some site, support playlist, and file upload.</br>
      if passed playlist url and passed it with "shuffle" or "random" as last argments, playlist queue will be shuffled.

    - `stop` :</br>Stop to the music currently playing(if there) and queue will be empty.

    - `volume` :</br>Set the music volume. range is 0.0 ~ 100.0.

    - `pause` :</br>Pause the music currently playing.

    - `resume` :</br>Resume the music currently playing.

    - `skip` :</br>Skip the music currently playing or specified number of songs from the queue.

    - `loop <on/off>` :</br>Enable/Disable loop the current playing song.

    - `current` :</br>Shows the info of the music currently playing.

    - `queue` :</br>Shows a list of songs in the queue. index is 0 first.
  </details>

## TODO

- [ ] add more command(?) (plan: `seek`, `shuffle` for queue, etc.)
- [ ] To avoid complexity, remove arguments from the command and split it into multiple commands(plan: The split of the `play` command)
- [ ] Add message when a command fails
- [ ] Allow role limits to be controlled by environment variables
- [ ] Faster video metadata acquisition *1
- [x] more refactoring
- [x] basic refactoring
- [x] remove all `unwrap()`
- [x] make unique queue system, not `builtin-queue`

*1:
> Currently, Sympho uses `youtube-dl` or` ffprobe` for files to get metadata such as video titles.</br>
This process can be very slow to spawn a child process, execute commands within that child process, and parse the results.</br>
That's fine for `ffprobe`, but not for` youtube-dl`.</br>
Currently, I am experimenting with using `pyo3` in the development environment and executing it directly from the` youtube-dl` module of Python.</br>
This will improve the retrieval of metadata.</br>
In the future, we'll need something like `youtube-dl` written in Rust :3</br>

## Advantages over other bots

- All written in Rust, so small single binary, blazing fast, and very small memory footprint
- Completely open source, and Easy to build (just set `DISCORD_TOKEN` env then run `cargo run --release`)
- Anyone can host Sympho on your server
- Sympho can play the uploaded audio file
- Depends on youtube-dl, but not limited to Youtube or Soundcloud URLs(For example, sympho also accept bilibili videos)
- There are no premium restrictions, don't waste your money
- The code isn't too dirty, so anyone can customize it like a addtional command, addtional features

## Dis-Advantages over other bots

- Missing some commands(?)
- Self-hosting costs
- Difficult to customize for people who can't Rust
- There is no such thing as an effect function for song
- There is no command to operation related to the queue(but i have plan)
- Unstable(e.g. there is a problem with the sound being played, or the sound at the start of playback is a little strange.)
- And... I'm the only one who claims the code isn't dirty QwQ
- I'M THE ONlY DEVELOPER FOR SYMPHO

## Require

- rust
- ffmpeg
- libopus
- youtube-dl
- ~~Python3 (for pyo3)~~(under development's feature)

see more information [Songbird's README#dependencies](https://github.com/serenity-rs/songbird#dependencies)

tested on Windows10 x64 and Ubuntu 20.04 with rust nightly 1.53 ~ 1.54

## Build and run

clone, set token to `DISCORD_TOKEN` env var, then `cargo build`, then run it.
