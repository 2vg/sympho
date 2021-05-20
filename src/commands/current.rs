use crate::define::*;
use crate::import::*;

#[command]
#[aliases("np", "nowplaying", "なうぷれ")]
#[only_in(guilds)]
#[description("Shows the info of the music currently playing.")]
async fn current(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = if let Some(g) = msg.guild(&ctx.cache).await {
        g
    } else {
        check_msg(
            msg.reply(
                ctx,
                "The bot could'nt get the guild information, please contact to developer. >_<!",
            )
            .await,
        );
        return Ok(());
    };
    let guild_id = guild.id;
    let manager = if let Some(m) = songbird::get(ctx).await {
        m
    } else {
        check_msg(
            msg.reply(
                ctx,
                "The bot have something problem, please contact to developer. >_<!",
            )
            .await,
        );
        return Ok(());
    };

    if let Some(_handler_lock) = manager.get(guild_id) {
        let data = ctx.data.read().await;
        if let Some(sympho_global_mutex) = data.get::<SymphoGlobal>() {
            let mut sympho_global = sympho_global_mutex.write().await;
            let sympho_data = sympho_global.entry(guild_id.0).or_insert(SymphoData {
                volume: 1.0,
                ..Default::default()
            });

            if let Some((track_handle, track_sympho)) = &sympho_data.current {
                say_track_with_embed(msg, ctx, track_handle, track_sympho).await;
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

async fn say_track_with_embed(
    msg: &Message,
    ctx: &Context,
    track_handle: &TrackHandle,
    _track_sympho: &TrackSympho,
) {
    let track_current_position = if let Ok(info) = track_handle.get_info().await {
        dur_to_hhmmss((*info).position)
    } else {
        "Unknown".to_string()
    };

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
                        "{} / {}",
                        track_current_position,
                        if let Some(dur) = track_handle.metadata().duration {
                            dur_to_hhmmss(dur)
                        } else {
                            "Unknown".to_string()
                        }
                    ));
                    if let Some(url) = &track_handle.metadata().source_url {
                        e.url(url);
                    }
                    if let Some(thumb_url) = &track_handle.metadata().thumbnail {
                        e.thumbnail(thumb_url);
                    }

                    e
                });

                m
            })
            .await,
    );
}
