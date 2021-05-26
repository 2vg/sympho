use crate::define::*;
use crate::import::*;

const SHUFFLE_WORDS: &[&str] = &["shuffle", "random"];

#[command]
#[aliases("p")]
#[only_in(guilds)]
#[description("Start to play music. supported some site, support playlist, file upload\nusage: <PREFIX>play https://youtube.com/watch?v=... or, play with file upload.\nif passed playlist url and passed it with "shuffle" or "random" as second argments, playlist queue will be shuffled.")]
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

    let enable_shuffle = if let Ok(shuffle) = args.single::<String>() {
        if SHUFFLE_WORDS.contains(&(shuffle.as_str())) {
            true
        } else {
            false
        }
    } else {
        false
    };

    if Url::parse(&url).is_err() {
        return Ok(());
    }

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

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let len = enqueue(ctx, guild_id.0, url.clone(), enable_shuffle).await;

        if len != 0 {
            check_msg(
                msg.reply(&ctx.http, format!("Added {} song to queue.", len))
                    .await,
            );

            dequeue(&mut handler, ctx, guild_id.0).await;
        }
    } else {
        check_msg(
            msg.reply(ctx, "The bot is not in a voice channel. >_<!")
                .await,
        );
    }

    Ok(())
}

pub async fn enqueue(ctx: &Context, key: u64, url: String, enable_shuffle: bool) -> usize {
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
                thumb: None,
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
                        let mut track_vec = Vec::new();
                        let mut list_count = 0usize;

                        for sv in entries {
                            let url = sv.url.unwrap_or("".to_string());
                            if url != "" {
                                let dur = if let Some(dur) = sv.duration {
                                    Duration::from_secs_f64(dur.as_f64().unwrap_or(0.0))
                                } else {
                                    Duration::new(0, 0)
                                };
                                track_vec.push(TrackSympho {
                                    url: format!("https://www.youtube.com/watch?v={}", url),
                                    title: sv.title,
                                    thumb: sv.thumbnail,
                                    duration: dur,
                                });
                                sympho_data.queue_duration += dur;
                                list_count += 1;
                            };
                        }

                        if enable_shuffle {
                            let mut rng = rand::thread_rng();
                            track_vec.shuffle(&mut rng);
                        }

                        sympho_data.queue.extend_from_slice(&track_vec);

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
                                thumb: yt_sv.thumbnail,
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

pub async fn dequeue(handler: &mut Call, ctx: &Context, key: u64) {
    let data = ctx.data.read().await;
    if let Some(sympho_global_mutex) = data.get::<SymphoGlobal>() {
        let mut sympho_global = sympho_global_mutex.write().await;
        let sympho_data = sympho_global.entry(key).or_insert(SymphoData {
            volume: 1.0,
            ..Default::default()
        });

        if sympho_data.queue.len() == 0 || sympho_data.current.is_some() {
            return;
        }

        let track_sympho = sympho_data.queue[0].clone();
        let url = track_sympho.url.clone();

        sympho_data.queue.remove(0);

        let source = if let Ok(source) = get_source(url).await {
            source
        } else {
            return;
        };

        sympho_data.queue_duration -= track_sympho.duration;

        sympho_data.current = Some((
            play_from_source(handler, source, sympho_data.volume),
            track_sympho,
        ));
    }
}
