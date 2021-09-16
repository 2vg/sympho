use crate::commands::*;
use crate::import::*;

// Global var
pub static SYMPHO_ICON: OnceCell<Mutex<String>> = OnceCell::new();
pub static SYMPHO_NAME: OnceCell<Mutex<String>> = OnceCell::new();
pub static SYMPHO_PREFIX: OnceCell<Mutex<String>> = OnceCell::new();

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

pub const EXCLUDE_HOOK: &[&str] = &[
    "help",
    "q",
    "queue",
    "current",
    "np",
    "nowplaying",
    "なうぷれ",
];

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

pub async fn get_source(url: String) -> Result<Input, ()> {
    if is_file_url(&url) {
        if let Ok(source) = Restartable::ffmpeg(url.clone(), false).await {
            //let mut source = Input::from(source);
            //if let Codec::Opus(ref mut opus) = source.kind {
            //    opus.allow_passthrough = false;
            //}
            Ok(source.into())
        } else {
            Err(())
        }
    } else {
        if let Ok(source) = Restartable::ytdl(url.clone(), false).await {
            //let mut source = Input::from(source);
            //if let Codec::Opus(ref mut opus) = source.kind {
            //    opus.allow_passthrough = false;
            //}
            Ok(source.into())
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
    ctx: &Context,
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

pub fn dur_to_hhmmss(dur: Duration) -> String {
    let secs = dur.as_secs();
    let seconds = secs % 60;
    let minutes = (secs / 60) % 60;
    let hours = (secs / 60) / 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

pub fn run_cmd(cmd: &str, args: &[&str], timeout: Option<Duration>) -> Result<Value> {
    let mut child = Command::new(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(args)
        .spawn()?;

    // Continually read from stdout so that it does not fill up with large output and hang forever.
    // We don't need to do this for stderr since only stdout has potentially giant JSON.
    let mut stdout = Vec::new();
    let child_stdout = child.stdout.take();
    std::io::copy(&mut child_stdout.unwrap(), &mut stdout)?;

    let exit_code = if let Some(timeout) = timeout {
        match child.wait_timeout(timeout)? {
            Some(status) => status,
            None => {
                child.kill()?;
                bail!("Process time out.");
            }
        }
    } else {
        child.wait()?
    };

    if exit_code.success() {
        Ok(serde_json::from_reader(stdout.as_slice())?)
    } else {
        let mut stderr = vec![];
        if let Some(mut reader) = child.stderr {
            reader.read_to_end(&mut stderr)?;
        }
        let stderr = String::from_utf8(stderr).unwrap_or_default();
        bail!(stderr);
    }
}

pub fn get_audio_file_info(url: &str) -> Result<(String, Duration)> {
    let info = run_cmd(
        "ffprobe",
        &[
            "-hide_banner",
            "-show_entries",
            "format_tags=TITLE:format=duration",
            "-of",
            "json",
            "-v",
            "quiet",
            url,
        ],
        Some(Duration::new(5, 0)),
    )?;
    let title = String::from(
        info["format"]["tags"]["TITLE"]
            .as_str()
            .unwrap_or("Unknown"),
    );
    let dur = Duration::from_secs_f64(
        info["format"]["duration"]
            .as_str()
            .unwrap_or("0.0")
            .parse::<f64>()?,
    );
    Ok((title, dur))
}
