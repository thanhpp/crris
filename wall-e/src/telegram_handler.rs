use teloxide::{
    prelude::*, requests::ResponseResult, types::Message, utils::command::BotCommands, Bot,
};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "snake_case",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "List commands")]
    Help,
    #[command(description = "Show balance")]
    ShowBalance,
    #[command(description = "Add balance", parse_with = "split")]
    AddBalance {
        user: String,
        op_1: String,
        amount_1: String,
        unit_1: String,
        op_2: String,
        amount_2: String,
        unit_2: String,
    },
}

pub async fn start(token: &str) {
    let b = Bot::new(token);

    b.set_my_commands(Command::bot_commands())
        .await
        .expect("set commands error");

    Command::repl(b, handle_message).await;
}

async fn handle_message(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::ShowBalance => bot.send_message(msg.chat.id, "show_balanace").await?,
        Command::AddBalance {
            user,
            op_1,
            amount_1,
            unit_1,
            op_2,
            amount_2,
            unit_2,
        } => {
            bot.send_message(
                msg.chat.id,
                format!("{user} ,{op_1}, {amount_1}, {unit_1}, {op_2}, {amount_2}, {unit_2} "),
            )
            .await?
        }
    };

    Ok(())
}
