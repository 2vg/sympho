use crate::commands::*;
use crate::import::*;

// Global const
pub const PREFIX: &'static str = "-";

// Global var
pub static mut SYMPHO_ICON: OnceCell<Mutex<String>> = OnceCell::new();
pub static mut SYMPHO_NAME: OnceCell<Mutex<String>> = OnceCell::new();

// Track Info
#[derive(Clone, Debug)]
pub struct TrackSympho {
    pub url: String,
    pub title: String,
    pub thumb: Option<String>,
    pub duration: Duration,
}

// Global Queue Struct that used in Sympho
#[derive(Debug, Default)]
pub struct SymphoData {
    pub current: Option<(TrackHandle, TrackSympho)>,
    pub volume: f32,
    pub queue: Vec<TrackSympho>,
    pub queue_duration: Duration,
}

// For Serenity's Global data
// Because it will share across threads, so need RwLock and Arc for sharing reference
pub struct SymphoGlobal;

impl TypeMapKey for SymphoGlobal {
    type Value = Arc<RwLock<HashMap<u64, SymphoData>>>;
}

// basic handler struct
// for now, only handle when bot connect
pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

// Serenity Command Group Struct
// command derive represents the command that Sympho has
#[group]
#[commands(
    help, join, leave, play, stop, volume, pause, resume, skip, looping, current, queue
)]
pub struct General;

pub fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

pub fn is_file_url(url: &str) -> bool {
    let url = if let Ok(url) = Url::parse(url) {
        url
    } else {
        return true;
    };
    Path::new(url.path()).extension().is_some()
}

pub async fn get_source(url: String) -> Result<Restartable, ()> {
    if is_file_url(&url) {
        if let Ok(source) = Restartable::ffmpeg(url.clone(), true).await {
            Ok(source)
        } else {
            Err(())
        }
    } else {
        if let Ok(source) = Restartable::ytdl(url.clone(), true).await {
            Ok(source)
        } else {
            Err(())
        }
    }
}

pub fn play_from_source(handler: &mut Call, src: Input, volume: f32) -> TrackHandle {
    let (mut track, track_handle) = create_player(src);

    track.set_volume(volume);

    handler.play(track);

    track_handle
}

pub fn has_dj_user(guild: &Guild, roles: &[RoleId]) -> bool {
    for role_id in roles {
        if let Some(role) = guild.roles.get(&role_id) {
            if role.name == "DJUser" {
                return true;
            }
        }
    }

    false
}

pub fn in_channel(guild: &Guild, msg: &Message) -> bool {
    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    match channel_id {
        Some(_) => true,
        None => false,
    }
}

pub async fn check_user_can_use_command(guild: &Guild, ctx: &Context, msg: &Message) -> bool {
    if !in_channel(guild, msg) {
        check_msg(msg.reply(ctx, "You are not in a voice channel. >_<!").await);
        return false;
    }
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
    true
}

// if bot playing music on other channel, return true.
pub async fn check_bot_using_at_other_chan(
    manager: &Songbird,
    guild: &Guild,
    msg: &Message,
    ctx: &Context
) -> bool {
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

                let data = ctx.data.read().await;
                if let Some(sympho_global_mutex) = data.get::<SymphoGlobal>() {
                    let mut sympho_global = sympho_global_mutex.write().await;
                    let sympho_data = sympho_global.entry(guild.id.0).or_insert(SymphoData {
                        volume: 1.0,
                        ..Default::default()
                    });

                    if let Some((track_handle, _track_sympho)) = &sympho_data.current {
                        if let Ok(info) = track_handle.get_info().await {
                            if info.playing == PlayMode::Play {
                                return true;
                            }
                        }
                    };
                }
            }
        }
    };

    false
}
