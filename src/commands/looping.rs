use crate::define::*;
use crate::import::*;

#[command]
#[aliases("loop")]
#[only_in(guilds)]
#[description("Enable/Disable loop the current playing song.\nusage: <PREFIX>loop on")]
async fn looping(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let looping = if let Ok(arg) = args.single::<String>() {
        if arg != "on" && arg != "off" {
            return Ok(());
        };
        arg
    } else {
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

            if let Some((current, _)) = &sympho_data.current {
                if looping == "on" {
                    if let Ok(_) = current.enable_loop() {
                        check_msg(
                            msg.reply(ctx, "Enabled loop the current playing song.")
                                .await,
                        );
                    }
                } else {
                    if let Ok(_) = current.disable_loop() {
                        check_msg(
                            msg.reply(ctx, "Disabled loop the current playing song.")
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
