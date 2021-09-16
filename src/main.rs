use sympho::define::*;
use sympho::import::*;

// Check user can use command
#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    if EXCLUDE_HOOK.contains(&command_name) {
        return true;
    }

    let guild = if let Some(g) = msg.guild(&ctx.cache) {
        g
    } else {
        return false;
    };
    let _guild_id = guild.id;

    if !check_user_can_use_command(&guild, ctx, msg).await {
        return false;
    };

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
        return false;
    };

    if check_bot_using_at_other_chan(&manager, &guild, msg, ctx).await {
        match command_name {
            "join" => {
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
                    return false;
                };

                check_msg(
                    msg.reply(
                        &ctx.http,
                        &format!("The bot is currently playing at {}", connect_to.mention()),
                    )
                    .await,
                );
            }
            _ => {}
        }
        return false;
    }

    true
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);

    if let Ok(bot_info) = http.get_current_application_info().await {
        SYMPHO_ICON.get_or_init(|| {
            if let Some(icon_url) = bot_info.icon.clone() {
                Mutex::new(format!(
                    "https://cdn.discordapp.com/app-icons/{}/{}.png",
                    bot_info.id.0, icon_url
                ))
            } else {
                Mutex::new("https://cdn.discordapp.com/embed/avatars/0.png".to_string())
            }
        });
        SYMPHO_NAME.get_or_init(|| Mutex::new(bot_info.name.clone()));
        SYMPHO_PREFIX
            .get_or_init(|| Mutex::new(env::var("SYMPHO_PREFIX").unwrap_or("!".to_string())));
    }

    let prefix = if let Ok(sympho_prefix) = SYMPHO_PREFIX
        .get_or_init(|| Mutex::new("!".to_string()))
        .lock()
    {
        (*sympho_prefix).clone()
    } else {
        "!".to_string()
    };

    let framework = StandardFramework::new()
        .configure(|c| c.prefix(&prefix))
        .before(before)
        .group(&GENERAL_GROUP);

    let songbird_config = Config::default()
        .crypto_mode(CryptoMode::Lite)
        .decode_mode(DecodeMode::Pass);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird_from_config(songbird_config)
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
