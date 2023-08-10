use chrono::Datelike;
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
            let date = get_date();
            let b_op = match BalanceOperation::new(&Command::AddBalance {
                user,
                op_1,
                amount_1,
                unit_1,
                op_2,
                amount_2,
                unit_2,
            }) {
                Err(e) => format!("{:#?}", e),
                Ok(b_op) => format!("{:#?}", b_op),
            };
            bot.send_message(msg.chat.id, format!("{date} {}", b_op))
                .await?
        }
    };

    Ok(())
}

fn get_date() -> String {
    let n = chrono::Local::now();
    format!("{}/{}/{}", n.day(), n.month(), n.year())
}

#[derive(Default, Debug)]
struct BalanceOperation {
    user: String,

    sign_positive_1: bool,
    amount_1: f64,
    unit_1: String,

    sign_positive_2: bool,
    amount_2: f64,
    unit_2: String,
}

impl BalanceOperation {
    fn new(c: &Command) -> anyhow::Result<BalanceOperation> {
        let valid_users = vec![String::from("tpp"), String::from("pch")];
        let valid_ops = vec![String::from("+"), String::from("-")];
        let valid_units = vec![String::from("vnd"), String::from("pl")];
        let mut b_op = BalanceOperation::default();

        match c {
            Command::AddBalance {
                user,
                op_1,
                amount_1,
                unit_1,
                op_2,
                amount_2,
                unit_2,
            } => {
                let user = user.to_lowercase();
                if !valid_users.contains(&user) {
                    return Err(anyhow::format_err!("invalid user: [{}]", user));
                }
                b_op.user = user;

                if !valid_ops.contains(op_1) {
                    return Err(anyhow::format_err!("invalid op 1: [{}]", op_1));
                }
                if op_1.as_str() == "+" {
                    b_op.sign_positive_1 = true;
                }

                b_op.amount_1 = Self::parse_amount(amount_1)?;

                let unit_1 = unit_1.to_lowercase();
                if !valid_units.contains(&unit_1) {
                    return Err(anyhow::format_err!("invalid unit 1: [{}]", unit_1));
                }
                b_op.unit_1 = unit_1;

                if !valid_ops.contains(op_2) {
                    return Err(anyhow::format_err!("invalid op 2: [{}]", op_2));
                }
                if op_2.as_str() == "+" {
                    b_op.sign_positive_2 = true;
                }

                b_op.amount_2 = Self::parse_amount(amount_2)?;

                let unit_2 = unit_2.to_lowercase();
                if !valid_units.contains(&unit_2) {
                    return Err(anyhow::format_err!("invalid unit 2: [{}]", unit_2));
                }
                b_op.unit_2 = unit_2;
            }
            _ => return Err(anyhow::format_err!("not add balance command")),
        };

        Ok(b_op)
    }

    fn parse_amount(amount: &str) -> anyhow::Result<f64> {
        let mut contain_tr = false;
        if amount.contains("tr") {
            contain_tr = true
        }
        let amount = amount.replace("tr", "");
        let mut amount = amount.parse::<f64>()?;
        if !contain_tr {
            return Ok(amount);
        }
        amount *= 1_000_000.0;
        Ok(amount)
    }
}
