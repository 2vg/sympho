use crate::define::*;
use crate::events::*;
use crate::import::*;

#[command]
#[only_in(guilds)]
#[description("Join the VC channel with the user who called !join.")]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = if let Some(g) = msg.guild(&ctx.cache) {
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

    let connect_to = if let Some(c) = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id)
    {
        c
    } else {
        check_msg(
                msg.reply(ctx, "The bot could'nt get the voice channel information, please contact to developer. >_<!")
                    .await,
            );
        return Ok(());
    };

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
