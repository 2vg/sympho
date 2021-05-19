use humantime::format_duration;
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
        channel::Message,
        gateway::Ready,
        guild::Guild,
        id::RoleId,
        misc::Mentionable,
        // prelude::ChannelId,
    },
    prelude::{TypeMap, TypeMapKey},
    Result as SerenityResult,
};
use songbird::{
    create_player,
    driver::{Config, CryptoMode, DecodeMode},
    input::{restartable::Restartable, Input},
    tracks::{PlayMode, TrackHandle},
    Call, Event, EventContext, EventHandler as VoiceEventHandler, SerenityInit, Songbird,
    TrackEvent,
};
use std::{
    collections::HashMap,
    env,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::RwLock;
use url::Url;
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

#[derive(Debug)]
struct TrackSympho {
    url: String,
    title: String,
    duration: Duration,
}

#[derive(Debug, Default)]
struct SymphoData {
    current: Option<TrackHandle>,
    volume: f32,
    queue: Vec<TrackSympho>,
    queue_duration: Duration,
}

struct SymphoGlobal;

impl TypeMapKey for SymphoGlobal {
    type Value = Arc<RwLock<HashMap<u64, SymphoData>>>;
}

static mut SYMPHO_ICON: OnceCell<Mutex<String>> = OnceCell::new();
static mut SYMPHO_NAME: OnceCell<Mutex<String>> = OnceCell::new();

const HELP_TEXT: &'static str = r"
```markdown
# made by uru(ururu#5687)

- !play <youtube, soundcloud url> : play music. supported single video, and playlist
- !loop <on or off> : enable/disable loop the current playing song
- !volume <1 - 100> : set the music volume
- !queue <0 - ?> : Shows a list of songs in the queue. index is 0 first.

- !join : Join the VC channel with the user who called !join
- !leave : leave from the current channel
- !current(or nowplaying) : shows the title of the music currently playing
- !skip : skip the music currently playing
- !pause : pause the music currently playing
- !resume : resume the music currently playing
- !stop : stop to the music currently playing(if there) and queue will be empty
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
#[commands(
    join, leave, play, current, queue, volume, skip, pause, resume, stop, looping, help
)]
struct General;

// event
struct TrackStartNotifier {
    data: Arc<serenity::prelude::RwLock<TypeMap>>,
    key: u64,
    // chan_id: ChannelId,
    // http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for TrackStartNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(&[(_state, track)]) = ctx {
            let data = self.data.read().await;
            if let Some(sympho_global_mutex) = data.get::<SymphoGlobal>() {
                let mut sympho_global = sympho_global_mutex.write().await;
                let sympho_data = sympho_global.entry(self.key).or_insert(SymphoData {
                    volume: 1.0,
                    ..Default::default()
                });
                let _ = track.set_volume(sympho_data.volume);
            }
        }

        None
    }
}

struct TrackEndNotifier {
    handler: Arc<serenity::prelude::Mutex<Call>>,
    data: Arc<serenity::prelude::RwLock<TypeMap>>,
    key: u64,
    // chan_id: ChannelId,
    // http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let mut handler = self.handler.lock().await;
        let data = self.data.read().await;

        if let Some(sympho_global_mutex) = data.get::<SymphoGlobal>() {
            let mut sympho_global = sympho_global_mutex.write().await;
            let sympho_data = sympho_global.entry(self.key).or_insert(SymphoData {
                volume: 1.0,
                ..Default::default()
            });

            if sympho_data.queue.len() != 0 {
                let url = sympho_data.queue[0].url.clone();

                let source = if let Ok(source) = get_source(url).await {
                    source
                } else {
                    return None;
                };

                sympho_data.queue_duration -= sympho_data.queue[0].duration;

                sympho_data.queue.remove(0);

                sympho_data.current = Some(play_from_source(
                    &mut handler,
                    source.into(),
                    sympho_data.volume,
                ));
            } else {
                sympho_data.current = None;
                return None;
            }
        }

        None
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);

    if let Ok(bot_info) = http.get_current_application_info().await {
        unsafe {
            SYMPHO_ICON.get_or_init(|| {
                Mutex::new(format!(
                    "https://cdn.discordapp.com/avatars/{}/{}.png",
                    bot_info.id.0,
                    bot_info.icon.clone().unwrap_or("0".to_string()).clone()
                ))
            });
            SYMPHO_NAME.get_or_init(|| Mutex::new(bot_info.name.clone()));
        }
    }

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird_with({
            let songbird = songbird::Songbird::serenity();

            songbird.set_config(Config {
                crypto_mode: CryptoMode::Normal,
                decode_mode: DecodeMode::Pass,
                preallocated_tracks: 2,
            });

            songbird
        })
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<SymphoGlobal>(Arc::new(RwLock::new(HashMap::default())));
    }

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

    if !check_user_can_use_command(&guild, ctx, msg).await {
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
            msg.reply(
                &ctx.http,
                &format!("The bot is currently playing at {}", connect_to.mention()),
            )
            .await,
        );

        return Ok(());
    }

    let (handle_lock, success) = manager.join(guild_id, connect_to).await;

    if let Ok(_channel) = success {
        // let chan_id = msg.channel_id;

        // let send_http = ctx.http.clone();

        let mut handle = handle_lock.lock().await;

        handle.add_global_event(
            Event::Track(TrackEvent::Play),
            TrackStartNotifier {
                data: ctx.data.clone(),
                key: guild_id.0,
                // chan_id,
                // http: send_http.clone(),
            },
        );

        handle.add_global_event(
            Event::Track(TrackEvent::End),
            TrackEndNotifier {
                handler: handle_lock.clone(),
                data: ctx.data.clone(),
                key: guild_id.0,
                // chan_id,
                // http: send_http.clone(),
            },
        );

        check_msg(
            msg.channel_id
                .say(&ctx.http, &format!("Joined {}", connect_to.mention()))
                .await,
        );
    } else {
        check_msg(msg.reply(&ctx.http, "Error joining the channel.").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if !check_user_can_use_command(&guild, ctx, msg).await {
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
            msg.reply(
                &ctx.http,
                &format!("The bot is currently playing at {}", connect_to.mention()),
            )
            .await,
        );

        return Ok(());
    }

    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        manager.remove(guild_id).await?;

        let data = ctx.data.read().await;
        if let Some(sympho_global_mutex) = data.get::<SymphoGlobal>() {
            let mut sympho_global = sympho_global_mutex.write().await;
            let sympho_data = sympho_global.entry(guild_id.0).or_insert(SymphoData {
                volume: 1.0,
                ..Default::default()
            });

            sympho_data.queue = Vec::default();
            sympho_data.queue_duration = Duration::default();

            if let Some(current) = &sympho_data.current {
                current.stop()?;
                sympho_data.current = None;
            }
        }
    }

    Ok(())
}

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.reply(&ctx.http, HELP_TEXT).await);

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
                msg.reply(
                    &ctx.http,
                    "Must provide a URL to a video or audio, or attachments",
                )
                .await,
            );

            return Ok(());
        }
    };

    if Url::parse(&url).is_err() {
        return Ok(());
    }

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if !check_user_can_use_command(&guild, ctx, msg).await {
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

        let len = enqueue(ctx, guild_id.0, url.clone()).await;

        check_msg(
            msg.reply(&ctx.http, format!("Added {} song to queue.", len))
                .await,
        );

        dequeue(&mut handler, ctx, guild_id.0).await;
    } else {
        check_msg(
            msg.reply(ctx, "The bot is not in a voice channel. >_<!")
                .await,
        );
    }

    Ok(())
}

#[command]
#[aliases("nowplaying")]
#[only_in(guilds)]
async fn current(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if !check_user_can_use_command(&guild, ctx, msg).await {
        return Ok(());
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if check_bot_using_at_other_chan(&manager, &guild, msg).await {
        return Ok(());
    }

    if let Some(_handler_lock) = manager.get(guild_id) {
        let data = ctx.data.read().await;
        if let Some(sympho_global_mutex) = data.get::<SymphoGlobal>() {
            let mut sympho_global = sympho_global_mutex.write().await;
            let sympho_data = sympho_global.entry(guild_id.0).or_insert(SymphoData {
                volume: 1.0,
                ..Default::default()
            });

            if let Some(track_handle) = &sympho_data.current {
                say_track_with_embed(msg, ctx, track_handle).await;
            } else {
                check_msg(msg.reply(&ctx.http, "No songs.").await);
            };
        }
    } else {
        check_msg(
            msg.reply(ctx, "The bot is not in a voice channel. >_<!")
                .await,
        );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn volume(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let volume = match args.single::<f32>() {
        Ok(vol) => {
            if vol >= 0.1 && vol <= 100.0 {
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

    if !check_user_can_use_command(&guild, ctx, msg).await {
        return Ok(());
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if check_bot_using_at_other_chan(&manager, &guild, msg).await {
        return Ok(());
    }

    let volume = volume / 100.0;

    if let Some(_handler_lock) = manager.get(guild_id) {
        let data = ctx.data.read().await;
        if let Some(sympho_global_mutex) = data.get::<SymphoGlobal>() {
            let mut sympho_global = sympho_global_mutex.write().await;
            let sympho_data = sympho_global.entry(guild_id.0).or_insert(SymphoData {
                volume: 1.0,
                ..Default::default()
            });

            sympho_data.volume = volume;

            if let Some(current) = &sympho_data.current {
                current.set_volume(volume)?;
            }
        }
    } else {
        check_msg(
            msg.reply(ctx, "The bot is not in a voice channel. >_<!")
                .await,
        );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn skip(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if !check_user_can_use_command(&guild, ctx, msg).await {
        return Ok(());
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if check_bot_using_at_other_chan(&manager, &guild, msg).await {
        return Ok(());
    }

    if let Some(_handler_lock) = manager.get(guild_id) {
        let data = ctx.data.read().await;
        if let Some(sympho_global_mutex) = data.get::<SymphoGlobal>() {
            let mut sympho_global = sympho_global_mutex.write().await;
            let sympho_data = sympho_global.entry(guild_id.0).or_insert(SymphoData {
                volume: 1.0,
                ..Default::default()
            });

            if let Some(current) = &sympho_data.current {
                current.stop()?;
                check_msg(msg.reply(&ctx.http, format!("Song skipped.")).await);
            }
        }
    } else {
        check_msg(
            msg.reply(ctx, "The bot is not in a voice channel. >_<!")
                .await,
        );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn pause(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if !check_user_can_use_command(&guild, ctx, msg).await {
        return Ok(());
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if check_bot_using_at_other_chan(&manager, &guild, msg).await {
        return Ok(());
    }

    if let Some(_handler_lock) = manager.get(guild_id) {
        let data = ctx.data.read().await;
        if let Some(sympho_global_mutex) = data.get::<SymphoGlobal>() {
            let mut sympho_global = sympho_global_mutex.write().await;
            let sympho_data = sympho_global.entry(guild_id.0).or_insert(SymphoData {
                volume: 1.0,
                ..Default::default()
            });

            if let Some(current) = &sympho_data.current {
                current.pause()?;
            }
        }
    } else {
        check_msg(
            msg.reply(ctx, "The bot is not in a voice channel. >_<!")
                .await,
        );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn resume(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if !check_user_can_use_command(&guild, ctx, msg).await {
        return Ok(());
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if check_bot_using_at_other_chan(&manager, &guild, msg).await {
        return Ok(());
    }

    if let Some(_handler_lock) = manager.get(guild_id) {
        let data = ctx.data.read().await;
        if let Some(sympho_global_mutex) = data.get::<SymphoGlobal>() {
            let mut sympho_global = sympho_global_mutex.write().await;
            let sympho_data = sympho_global.entry(guild_id.0).or_insert(SymphoData {
                volume: 1.0,
                ..Default::default()
            });

            if let Some(current) = &sympho_data.current {
                current.play()?;
            }
        }
    } else {
        check_msg(
            msg.reply(ctx, "The bot is not in a voice channel. >_<!")
                .await,
        );
    }

    Ok(())
}

#[command]
#[aliases("loop")]
#[only_in(guilds)]
async fn looping(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let looping = if let Ok(arg) = args.single::<String>() {
        if arg != "on" && arg != "off" {
            return Ok(());
        };
        arg
    } else {
        return Ok(());
    };

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if !check_user_can_use_command(&guild, ctx, msg).await {
        return Ok(());
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if check_bot_using_at_other_chan(&manager, &guild, msg).await {
        return Ok(());
    }

    if let Some(_handler_lock) = manager.get(guild_id) {
        let data = ctx.data.read().await;
        if let Some(sympho_global_mutex) = data.get::<SymphoGlobal>() {
            let mut sympho_global = sympho_global_mutex.write().await;
            let sympho_data = sympho_global.entry(guild_id.0).or_insert(SymphoData {
                volume: 1.0,
                ..Default::default()
            });

            if let Some(current) = &sympho_data.current {
                if looping == "on" {
                    if let Ok(_) = current.enable_loop() {
                        check_msg(
                            msg.reply(ctx, "Enabled loop the current playing song,")
                                .await,
                        );
                    }
                } else {
                    if let Ok(_) = current.disable_loop() {
                        check_msg(
                            msg.reply(ctx, "Disabled loop the current playing song,")
                                .await,
                        );
                    }
                }
            }
        }
    } else {
        check_msg(
            msg.reply(ctx, "The bot is not in a voice channel. >_<!")
                .await,
        );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn stop(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if !check_user_can_use_command(&guild, ctx, msg).await {
        return Ok(());
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if check_bot_using_at_other_chan(&manager, &guild, msg).await {
        return Ok(());
    }

    if let Some(_handler_lock) = manager.get(guild_id) {
        let data = ctx.data.read().await;
        if let Some(sympho_global_mutex) = data.get::<SymphoGlobal>() {
            let mut sympho_global = sympho_global_mutex.write().await;
            let sympho_data = sympho_global.entry(guild_id.0).or_insert(SymphoData {
                volume: 1.0,
                ..Default::default()
            });

            sympho_data.queue = Vec::default();
            sympho_data.queue_duration = Duration::default();

            if let Some(current) = &sympho_data.current {
                current.stop()?;
                sympho_data.current = None;
            }
        }

        check_msg(msg.reply(&ctx.http, "Stopped, queue was cleared.").await);
    } else {
        check_msg(
            msg.reply(ctx, "The bot is not in a voice channel. >_<!")
                .await,
        );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn queue(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let start_queue_index = if let Ok(arg) = args.single::<usize>() {
        arg
    } else {
        0
    };

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if !check_user_can_use_command(&guild, ctx, msg).await {
        return Ok(());
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if check_bot_using_at_other_chan(&manager, &guild, msg).await {
        return Ok(());
    }

    if let Some(_handler_lock) = manager.get(guild_id) {
        let data = ctx.data.read().await;
        if let Some(sympho_global_mutex) = data.get::<SymphoGlobal>() {
            let mut sympho_global = sympho_global_mutex.write().await;
            let sympho_data = sympho_global.entry(guild_id.0).or_insert(SymphoData {
                volume: 1.0,
                ..Default::default()
            });

            if sympho_data.queue.len() == 0 {
                check_msg(msg.reply(ctx, "Queue is empty.").await);

                return Ok(());
            }

            if start_queue_index >= sympho_data.queue.len() {
                return Ok(());
            }

            say_queue_with_embed(msg, ctx, sympho_data, start_queue_index).await;
        }
    } else {
        check_msg(
            msg.reply(ctx, "The bot is not in a voice channel. >_<!")
                .await,
        );
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

async fn enqueue(ctx: &Context, key: u64, url: String) -> usize {
    let data = ctx.data.read().await;
    if let Some(sympho_global_mutex) = data.get::<SymphoGlobal>() {
        let mut sympho_global = sympho_global_mutex.write().await;
        let sympho_data = sympho_global.entry(key).or_insert(SymphoData {
            volume: 1.0,
            ..Default::default()
        });

        if is_file_url(&url) {
            let dur = Duration::new(0, 0);
            sympho_data.queue.push(TrackSympho {
                url: url.clone(),
                title: "Unknown".to_string(),
                duration: dur,
            });
            sympho_data.queue_duration += dur;
            return 1;
        } else {
            let output = YoutubeDl::new(&url)
                .flat_playlist(true)
                .socket_timeout("3")
                .run();

            if let Ok(yt) = output {
                match yt {
                    YoutubeDlOutput::Playlist(yt_pl) => {
                        let entries = yt_pl.entries.unwrap_or(vec![]);
                        let mut list_count = 0usize;
                        for sv in entries {
                            let url = sv.url.unwrap_or("".to_string());
                            if url != "" {
                                let dur = if let Some(dur) = sv.duration {
                                    Duration::from_secs_f64(dur.as_f64().unwrap_or(0.0))
                                } else {
                                    Duration::new(0, 0)
                                };
                                sympho_data.queue.push(TrackSympho {
                                    url: url.clone(),
                                    title: sv.title,
                                    duration: dur,
                                });
                                sympho_data.queue_duration += dur;
                                list_count += 1;
                            };
                        }
                        return list_count;
                    }
                    YoutubeDlOutput::SingleVideo(yt_sv) => {
                        let url = yt_sv.webpage_url.unwrap_or("".to_string());
                        if url != "" {
                            let dur = if let Some(dur) = yt_sv.duration {
                                Duration::from_secs_f64(dur.as_f64().unwrap_or(0.0))
                            } else {
                                Duration::new(0, 0)
                            };
                            sympho_data.queue.push(TrackSympho {
                                url: url.clone(),
                                title: yt_sv.title,
                                duration: dur,
                            });
                            sympho_data.queue_duration += dur;
                            return 1;
                        };
                    }
                };
            }
        }
    }

    0
}

async fn dequeue(handler: &mut Call, ctx: &Context, key: u64) {
    let data = ctx.data.write().await;
    if let Some(sympho_global_mutex) = data.get::<SymphoGlobal>() {
        let mut sympho_global = sympho_global_mutex.write().await;
        let sympho_data = sympho_global.entry(key).or_insert(SymphoData {
            volume: 1.0,
            ..Default::default()
        });

        if sympho_data.queue.len() == 0 || sympho_data.current.is_some() {
            return;
        }

        let url = sympho_data.queue[0].url.clone();

        let source = if let Ok(source) = get_source(url).await {
            source
        } else {
            return;
        };

        sympho_data.queue_duration -= sympho_data.queue[0].duration;

        sympho_data.queue.remove(0);

        sympho_data.current = Some(play_from_source(handler, source.into(), sympho_data.volume));
    }
}

fn is_file_url(url: &str) -> bool {
    let url = Url::parse(url).unwrap();
    Path::new(url.path()).extension().is_some()
}

async fn get_source(url: String) -> Result<Restartable, ()> {
    if is_file_url(&url) {
        if let Ok(source) = Restartable::ffmpeg(url.clone(), false).await {
            Ok(source)
        } else {
            Err(())
        }
    } else {
        if let Ok(source) = Restartable::ytdl(url.clone(), false).await {
            Ok(source)
        } else {
            Err(())
        }
    }
}

fn play_from_source(handler: &mut Call, src: Input, volume: f32) -> TrackHandle {
    let (mut track, track_handle) = create_player(src);

    track.set_volume(volume);

    handler.play(track);

    track_handle
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

async fn check_user_can_use_command(guild: &Guild, ctx: &Context, msg: &Message) -> bool {
    if let Some(member) = &msg.member.as_ref() {
        if !has_dj_user(guild, &member.roles) {
            check_msg(
                msg.reply(ctx, "You don't have the role of `DJUser`. >_<!")
                    .await,
            );
            return false;
        }
    } else {
        return false;
    }
    if !in_channel(guild, msg) {
        check_msg(msg.reply(ctx, "You are not in a voice channel. >_<!").await);
        return false;
    }
    true
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

                if let Some(current) = handler.queue().current() {
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

async fn say_track_with_embed(msg: &Message, ctx: &Context, track_handle: &TrackHandle) {
    check_msg(
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.content("_nowplaying ♡:_");
                m.embed(|e| {
                    e.author(|a| {
                        if let Ok(icon) = unsafe {
                            SYMPHO_ICON.get_or_init(|| {
                                Mutex::new(
                                    "https://cdn.discordapp.com/embed/avatars/0.png".to_string(),
                                )
                            })
                        }
                        .lock()
                        {
                            a.icon_url(icon);
                        }

                        if let Ok(name) =
                            unsafe { SYMPHO_NAME.get_or_init(|| Mutex::new("Sympho".to_string())) }
                                .lock()
                        {
                            a.name(name);
                        }

                        a.url("https://github.com/2vg/sympho");

                        a
                    });
                    e.title(
                        track_handle
                            .metadata()
                            .title
                            .as_ref()
                            .unwrap_or(&"Unknown".to_string()),
                    );
                    e.description(&format!(
                        "length: {}",
                        if let Some(dur) = track_handle.metadata().duration {
                            format_duration(dur).to_string()
                        } else {
                            "Unknown".to_string()
                        }
                    ));
                    if let Some(url) = &track_handle.metadata().source_url {
                        e.url(url);
                    }
                    if let Some(thumb_url) = &track_handle.metadata().thumbnail {
                        if thumb_url.starts_with("https://") {
                            e.thumbnail(thumb_url);
                        }
                    }

                    e
                });

                m
            })
            .await,
    );
}

async fn say_queue_with_embed(
    msg: &Message,
    ctx: &Context,
    queue: &SymphoData,
    start_queue_index: usize,
) {
    let queue_len = queue.queue.len();
    let queue_slice = &queue.queue[start_queue_index..{
        if queue_len - start_queue_index > 10 {
            start_queue_index + 10
        } else {
            queue_len
        }
    }];

    let titles = queue_slice
        .iter()
        .enumerate()
        .fold(String::new(), |mut str, (i, q)| {
            str += &format!("{}: ", start_queue_index + i + 1);
            str += &format!("[{}]({})\n", &q.title, &q.url);
            str
        });

    let durations = queue_slice.iter().fold(String::new(), |mut str, q| {
        str += &format!("{}\n", format_duration(q.duration));
        str
    });

    check_msg(
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.author(|a| {
                        if let Ok(icon) = unsafe {
                            SYMPHO_ICON.get_or_init(|| {
                                Mutex::new(
                                    "https://cdn.discordapp.com/embed/avatars/0.png".to_string(),
                                )
                            })
                        }
                        .lock()
                        {
                            a.icon_url(icon);
                        }

                        if let Ok(name) =
                            unsafe { SYMPHO_NAME.get_or_init(|| Mutex::new("Sympho".to_string())) }
                                .lock()
                        {
                            a.name(name);
                        }

                        a.url("https://github.com/2vg/sympho");

                        a
                    });
                    e.title("Current queue information");
                    e.description(format!(
                        "Queue length: {}\nQueue Duration: {}",
                        queue.queue.len(),
                        format_duration(queue.queue_duration)
                    ));
                    e.field("Song title", titles, true);
                    e.field("Song Length", durations, true);
                    e
                });

                m
            })
            .await,
    );
}
