use crate::define::*;
use crate::import::*;

#[command]
#[aliases("v", "vol")]
#[only_in(guilds)]
#[description("Set the music volume. range is 0.0 ~ 100.0.\nusage: <PREFIX>volume 12.8")]
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

            if let Some((current, _)) = &sympho_data.current {
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
