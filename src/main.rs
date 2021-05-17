use std::{
    env,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use once_cell::sync::OnceCell;

use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{
        standard::{
            macros::{command, group},
            Args, CommandResult,
        },
        StandardFramework,
    },
    http::Http,
    model::{
        channel::Message, gateway::Ready, guild::Guild, id::RoleId, misc::Mentionable,
        prelude::ChannelId,
    },
    Result as SerenityResult,
};

use songbird::{
    create_player,
    input::{restartable::Restartable, Input},
    tracks::PlayMode,
    Call, Event, EventContext, EventHandler as VoiceEventHandler, SerenityInit, Songbird,
    TrackEvent,
};

use youtube_dl::{YoutubeDl, YoutubeDlOutput};

//static mut GLOBAL_TRACK: OnceCell<Mutex<TrackHandle>> = OnceCell::new();
static mut GLOBAL_QUEUE: OnceCell<Mutex<Vec<String>>> = OnceCell::new();
static mut GLOBAL_VOLUME: OnceCell<Mutex<f32>> = OnceCell::new();

const HELP_TEXT: &'static str = r"
```
made by uru(ururu#5687)

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
";

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(join, leave, play, current, volume, skip, pause, resume, stop, help)]
struct General;

// event
struct TrackStartNotifier {
    chan_id: ChannelId,
    http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for TrackStartNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(&[(_state, track)]) = ctx {
            let _ = track.set_volume(unsafe { *GLOBAL_VOLUME.get().unwrap().lock().unwrap() });
        }

        None
    }
}

struct TrackEndNotifier {
    handler: Arc<serenity::prelude::Mutex<Call>>,
    chan_id: ChannelId,
    http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(_track_list) = ctx {
            let volume = unsafe { *GLOBAL_VOLUME.get().unwrap().lock().unwrap() };
            let url = unsafe {
                let q = GLOBAL_QUEUE.get().unwrap();
                let mut v = q.lock().unwrap();

                if v.len() > 0 {
                    let temp = v[0].clone();
                    v.remove(0);
                    temp
                } else {
                    return None;
                }
            };

            let source = if let Ok(source) = Restartable::ytdl(url.clone(), true).await {
                source
            } else if let Ok(source) = Restartable::ffmpeg(url.clone(), true).await {
                source
            } else {
                return None;
            };

            let mut handler = self.handler.lock().await;
            play_from_source(&mut handler, source.into());
        }

        None
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    unsafe {
        GLOBAL_QUEUE.get_or_init(|| Mutex::new(vec![]));

        GLOBAL_VOLUME.get_or_init(|| Mutex::new(1.0));
    }

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Err creating client");

    let _ = client
        .start()
        .await
        .map_err(|why| println!("Client ended: {:?}", why));
}

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if !check_user_can_use_command(&guild, msg) {
        check_msg(msg.reply(ctx, "You are not in a voice channel >_<!").await);
        return Ok(());
    };

    let connect_to = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id)
        .unwrap();

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if check_bot_using_at_other_chan(&manager, &guild, &msg).await {
        check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    &format!("The bot is currently playing at {}", connect_to.mention()),
                )
                .await,
        );

        return Ok(());
    }

    let (handle_lock, success) = manager.join(guild_id, connect_to).await;

    if let Ok(_channel) = success {
        let chan_id = msg.channel_id;

        let send_http = ctx.http.clone();

        let mut handle = handle_lock.lock().await;
        let queue = handle.queue();
        let _ = queue.pause();

        handle.add_global_event(
            Event::Track(TrackEvent::Play),
            TrackStartNotifier {
                chan_id,
                http: send_http.clone(),
            },
        );

        handle.add_global_event(
            Event::Track(TrackEvent::End),
            TrackEndNotifier {
                handler: handle_lock.clone(),
                chan_id,
                http: send_http.clone(),
            },
        );

        check_msg(
            msg.channel_id
                .say(&ctx.http, &format!("Joined {}", connect_to.mention()))
                .await,
        );
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Error joining the channel")
                .await,
        );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if !check_user_can_use_command(&guild, msg) {
        check_msg(msg.reply(ctx, "You are not in a voice channel >_<!").await);
        return Ok(());
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let connect_to = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id)
        .unwrap();

    if check_bot_using_at_other_chan(&manager, &guild, msg).await {
        check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    &format!("The bot is currently playing at {}", connect_to.mention()),
                )
                .await,
        );

        return Ok(());
    }

    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        let _ = manager.remove(guild_id).await;

        if let Some(handler_lock) = manager.get(guild_id) {
            unsafe {
                let q = GLOBAL_QUEUE.get().unwrap();
                let mut v = q.lock().unwrap();
                *v = vec![];
            }

            let handler = handler_lock.lock().await;
            let queue = handler.queue();
            let _ = queue.stop();
        };
    }

    Ok(())
}

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, HELP_TEXT).await);

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = if let Ok(url) = args.single::<String>() {
        url
    } else {
        if &msg.attachments.len() != &0 {
            (&msg).attachments[0].url.clone()
        } else {
            check_msg(
                msg.channel_id
                    .say(
                        &ctx.http,
                        "Must provide a URL to a video or audio, or attachments",
                    )
                    .await,
            );

            return Ok(());
        }
    };

    if !url.starts_with("http") {
        return Ok(());
    }

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if !check_user_can_use_command(&guild, msg) {
        check_msg(msg.reply(ctx, "You are not in a voice channel >_<!").await);
        return Ok(());
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if check_bot_using_at_other_chan(&manager, &guild, msg).await {
        return Ok(());
    }

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let urls = extract_yt(&url);
        let len = urls.len();

        let url = unsafe {
            let q = GLOBAL_QUEUE.get().unwrap();
            let mut v = q.lock().unwrap();
            v.extend_from_slice(&urls);

            if v.len() > 0 {
                let temp = v[0].clone();
                v.remove(0);
                temp
            } else {
                return Ok(());
            }
        };

        let source = if let Ok(source) = Restartable::ytdl(url.clone(), true).await {
            source
        } else if let Ok(source) = Restartable::ffmpeg(url.clone(), true).await {
            source
        } else {
            return Ok(());
        };

        play_from_source(&mut handler, source.into());

        check_msg(
            msg.channel_id
                .say(&ctx.http, format!("Added {} song to queue :3", len))
                .await,
        );
    } else {
        check_msg(msg.reply(ctx, "You are not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn current(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if !check_user_can_use_command(&guild, msg) {
        check_msg(msg.reply(ctx, "You are not in a voice channel >_<!").await);
        return Ok(());
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if check_bot_using_at_other_chan(&manager, &guild, msg).await {
        return Ok(());
    }

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let current = queue.current();

        match current {
            Some(song) => {
                if let Some(title) = &song.metadata().title {
                    check_msg(
                        msg.channel_id
                            .say(&ctx.http, format!("Current track: {}", title))
                            .await,
                    );
                } else {
                    check_msg(
                        msg.channel_id
                            .say(&ctx.http, "Current track have no title.")
                            .await,
                    );
                }
            }
            None => {
                check_msg(msg.channel_id.say(&ctx.http, "No songs.").await);
            }
        };
    } else {
        check_msg(msg.reply(ctx, "You are not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn volume(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let volume = match args.single::<i64>() {
        Ok(vol) => {
            if vol >= 1 && vol <= 100 {
                vol
            } else {
                return Ok(());
            }
        }
        Err(_) => {
            return Ok(());
        }
    };

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if !check_user_can_use_command(&guild, msg) {
        check_msg(msg.reply(ctx, "You are not in a voice channel >_<!").await);
        return Ok(());
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if check_bot_using_at_other_chan(&manager, &guild, msg).await {
        return Ok(());
    }

    let volume = volume as f32 / 100.0;

    unsafe {
        let g_volume_mutex = GLOBAL_VOLUME.get().unwrap();
        let mut g_volume = g_volume_mutex.lock().unwrap();
        *g_volume = volume;
    }

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let current = queue.current();

        match current {
            Some(song) => {
                song.set_volume(volume)?;
            }
            None => {}
        };
    } else {
        check_msg(msg.reply(ctx, "You are not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn skip(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if !check_user_can_use_command(&guild, msg) {
        check_msg(msg.reply(ctx, "You are not in a voice channel >_<!").await);
        return Ok(());
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if check_bot_using_at_other_chan(&manager, &guild, msg).await {
        return Ok(());
    }

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.skip();

        let url = unsafe {
            let q = GLOBAL_QUEUE.get().unwrap();
            let mut v = q.lock().unwrap();

            if v.len() > 0 {
                let temp = v[0].clone();
                v.remove(0);
                temp
            } else {
                return Ok(());
            }
        };

        let source = if let Ok(source) = Restartable::ytdl(url.clone(), true).await {
            source
        } else if let Ok(source) = Restartable::ffmpeg(url.clone(), true).await {
            source
        } else {
            return Ok(());
        };

        play_from_source(&mut handler, source.into());

        check_msg(
            msg.channel_id
                .say(&ctx.http, format!("Song skipped."))
                .await,
        );
    } else {
        check_msg(msg.reply(ctx, "You are not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn pause(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if !check_user_can_use_command(&guild, msg) {
        check_msg(msg.reply(ctx, "You are not in a voice channel >_<!").await);
        return Ok(());
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if check_bot_using_at_other_chan(&manager, &guild, msg).await {
        return Ok(());
    }

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.pause();
    } else {
        check_msg(msg.reply(ctx, "You are not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn resume(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if !check_user_can_use_command(&guild, msg) {
        check_msg(msg.reply(ctx, "You are not in a voice channel >_<!").await);
        return Ok(());
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if check_bot_using_at_other_chan(&manager, &guild, msg).await {
        return Ok(());
    }

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.resume();
    } else {
        check_msg(msg.reply(ctx, "You are not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn stop(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if !check_user_can_use_command(&guild, msg) {
        check_msg(msg.reply(ctx, "You are not in a voice channel >_<!").await);
        return Ok(());
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if check_bot_using_at_other_chan(&manager, &guild, msg).await {
        return Ok(());
    }

    if let Some(handler_lock) = manager.get(guild_id) {
        unsafe {
            let q = GLOBAL_QUEUE.get().unwrap();
            let mut v = q.lock().unwrap();
            *v = vec![];
        }

        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.stop();

        check_msg(
            msg.channel_id
                .say(&ctx.http, "stopped, queue was cleared :3")
                .await,
        );
    } else {
        check_msg(msg.reply(ctx, "You are not in a voice channel").await);
    }

    Ok(())
}

/*
 * utility functions
 */

fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

fn extract_yt(url: &str) -> Vec<String> {
    let output = YoutubeDl::new(url)
        .flat_playlist(true)
        .socket_timeout("3")
        .run();

    if let Ok(yt) = output {
        match yt {
            YoutubeDlOutput::Playlist(yt_pl) => {
                let entries = yt_pl.entries.unwrap_or(vec![]);
                let mut v = Vec::with_capacity(*&entries.len());
                for sv in entries {
                    let url = sv.url.unwrap_or("".to_string());
                    if url != "" {
                        v.push(format!("https://www.youtube.com/watch?v={}", url));
                    };
                }
                return v;
            }
            YoutubeDlOutput::SingleVideo(yt_sv) => {
                let url = yt_sv.webpage_url.unwrap_or("".to_string());
                if url != "" {
                    return vec![url];
                };
                return vec![];
            }
        };
    } else {
        return vec![url.to_string()];
    }
}

fn play_from_source(handler: &mut Call, src: Input) {
    let g_vloume = unsafe { GLOBAL_VOLUME.get().unwrap().lock().unwrap() };

    let (mut track, _) = create_player(src);

    track.set_volume(*g_vloume);

    handler.enqueue(track);
}

fn has_dj_user(guild: &Guild, roles: &[RoleId]) -> bool {
    for role_id in roles {
        if let Some(role) = guild.roles.get(&role_id) {
            if role.name == "DJUser" {
                return true;
            }
        }
    }

    false
}

fn in_channel(guild: &Guild, msg: &Message) -> bool {
    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    match channel_id {
        Some(_) => true,
        None => false,
    }
}

fn check_user_can_use_command(guild: &Guild, msg: &Message) -> bool {
    return has_dj_user(guild, &msg.member.as_ref().unwrap().roles) && in_channel(guild, msg);
}

// if bot playing music on other channel, return true.
async fn check_bot_using_at_other_chan(manager: &Songbird, guild: &Guild, msg: &Message) -> bool {
    if let Some(channel_id) = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id)
    {
        if let Some(handler_lock) = manager.get(guild.id) {
            let handler = handler_lock.lock().await;

            if let Some(bot_channel_id) = handler.current_channel() {
                if channel_id.0 == bot_channel_id.0 {
                    return false;
                }

                let queue = handler.queue();
                let current = queue.current();

                if let Some(current) = queue.current() {
                    if let Ok(info) = current.get_info().await {
                        if info.playing == PlayMode::Play {
                            return true;
                        }
                    }
                }
            }
        }
    };

    false
}
