use crate::define::*;
use crate::import::*;

#[command]
#[only_in(guilds)]
#[description("THIS.\nusage: <PREFIX>help or <PREFIX>help <command name>")]
async fn help(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(arg) = args.single::<String>() {
        say_help_with_embed(msg, ctx, &arg).await;
    } else {
        say_help_list_with_embed(msg, ctx).await;
    };

    Ok(())
}

async fn say_help_with_embed(msg: &Message, ctx: &Context, cmd_name: &str) {
    let cmd_group = &GENERAL_GROUP.options;

    for cmd in cmd_group.commands {
        if cmd
            .options
            .names
            .iter()
            .find(|&&name| name == cmd_name)
            .is_some()
        {
            check_msg(
                msg.channel_id
                    .send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.author(|a| {
                                if let Ok(icon) = unsafe {
                                    SYMPHO_ICON.get_or_init(|| {
                                        Mutex::new(
                                            "https://cdn.discordapp.com/embed/avatars/0.png"
                                                .to_string(),
                                        )
                                    })
                                }
                                .lock()
                                {
                                    a.icon_url(icon);
                                }
                                if let Ok(name) = unsafe {
                                    SYMPHO_NAME.get_or_init(|| Mutex::new("Sympho".to_string()))
                                }
                                .lock()
                                {
                                    a.name(name);
                                }
                                a.url("https://github.com/2vg/sympho");
                                a
                            });
                            e.title(format!("{}{}", PREFIX, cmd_name));
                            e.description(cmd.options.desc.unwrap_or("description is empty"));
                            if cmd.options.names.len() > 1 {
                                e.field(
                                    "Aliases",
                                    cmd.options
                                        .names
                                        .iter()
                                        .skip(1)
                                        .map(|s| format!("{}{}", PREFIX, s))
                                        .collect::<Vec<_>>()
                                        .join("\n"),
                                    false,
                                );
                            }
                            e
                        });
                        m
                    })
                    .await,
            );
        }
    }
}

async fn say_help_list_with_embed(msg: &Message, ctx: &Context) {
    let cmd_group = &GENERAL_GROUP.options;

    let cmds = cmd_group
        .commands
        .iter()
        .fold(String::new(), |mut str, cmd| {
            str += &format!("{}{}\n", PREFIX, cmd.options.names[0]);
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
                    e.description(
                        [
                            "This bot made by ururu#5687.",
                            "Please contact to him if there problem!",
                            "If want to know about the each command, type !help <command name>",
                        ]
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join("\n"),
                    );

                    e.field("Command List", cmds, true);
                    e
                });
                m
            })
            .await,
    );
}
