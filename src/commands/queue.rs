use crate::define::*;
use crate::import::*;

#[command]
#[aliases("q")]
#[only_in(guilds)]
#[description("Shows a list of songs in the queue. index is 0 first.\nusage: <PREFIX>queue 0")]
async fn queue(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let start_queue_index = if let Ok(arg) = args.single::<usize>() {
        arg
    } else {
        0
    };

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
