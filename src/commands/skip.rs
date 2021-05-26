use crate::define::*;
use crate::import::*;

#[command]
#[only_in(guilds)]
#[description("Skip the music currently playing or specified number of songs from the queue.")]
async fn skip(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let (start, end) = if args.len() == 0 {
        (0, 0)
    } else if args.len() == 1 {
        if let Ok(arg) = args.single::<usize>() {
            (arg, 0)
        } else {
            return Ok(());
        }
    } else if args.len() == 2 {
        if let Ok(arg) = args.single::<usize>().and_then(|first_arg| {
            args.single::<usize>()
                .and_then(|second_arg| Ok((first_arg, second_arg)))
        }) {
            if arg.0 < 1 || arg.1 < 2 || arg.0 >= arg.1 {
                return Ok(());
            };
            arg
        } else {
            return Ok(());
        }
    } else {
        check_msg(
            msg.reply(
                ctx,
                "example usage: <PREFIX>skip <no number or 0> -> skip current playing song.\n
                 example usage: <PREFIX>skip 5 -> skip No.5 song.\n
                 example usage: <PREFIX>skip 1 10 -> skip No.1 ~ N0.10 songs on queue.",
            )
            .await,
        );
        return Ok(());
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

            if args.len() == 0 {
                if let Some((current, _)) = &sympho_data.current {
                    current.stop()?;
                    check_msg(
                        msg.reply(&ctx.http, format!("Currently playing song skipped."))
                            .await,
                    );
                }
            } else if args.len() == 1 {
                let queue_len = sympho_data.queue.len();

                if start < queue_len {
                    if start == 0 {
                        if let Some((current, _)) = &sympho_data.current {
                            current.stop()?;
                            check_msg(
                                msg.reply(&ctx.http, format!("Currently playing song skipped."))
                                    .await,
                            );
                        }
                    } else {
                        sympho_data.queue.drain(start - 1..start);
                        check_msg(
                            msg.reply(&ctx.http, format!("No.{} song skipped from queue.", start))
                                .await,
                        );
                    }
                }
            } else if args.len() == 2 {
                let queue_len = sympho_data.queue.len();

                if start < queue_len && end < queue_len {
                    sympho_data.queue.drain(start - 1..end);
                    check_msg(
                        msg.reply(
                            &ctx.http,
                            format!("No.{} - No.{} song skipped from queue.", start, end),
                        )
                        .await,
                    );
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
