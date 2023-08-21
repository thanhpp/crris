use chrono::Datelike;
use std::fs;
use teloxide::{
    dispatching::UpdateFilterExt, prelude::*, types::Message, utils::command::BotCommands, Bot,
};

use tokio::signal::unix::{signal, SignalKind};

mod gg_sheet;

const TELEGRAM_BOT_TOKEN_FILE: &str = "telegram_t_wall_e_bot_token";
const GOOGLE_SECRET_FILE: &str = "/home/thanhpp/.secrets/ggs_private_key.json";
const GOOGLE_SHEET_ID: &str = "1MKqvQ4tQiw0pk5LFlqW3CcZpOTzH5k8r6W9cwCSS1u8";

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "snake_case",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "List commands")]
    Help,
    #[command(description = "Show balance")]
    ShowBalance,
    #[command(description = "Add balance")]
    AddBalance(String),
}

#[tokio::main]
async fn main() {
    // init google sheet
    let ggs_client = gg_sheet::GgsClient::new(GOOGLE_SECRET_FILE, GOOGLE_SHEET_ID)
        .await
        .expect("set up ggs client error");

    ggs_client
        .read_range("Sheet1!D2")
        .await
        .expect("read range error");

    let r = ggs_client
        .find_empty_row("Sheet1!A:A")
        .await
        .expect("find empty range error");
    println!("row {}", r);

    // init telegram
    let data = fs::read_to_string(TELEGRAM_BOT_TOKEN_FILE).expect("read telegram token file error");
    let tele_bot = Bot::new(data);
    tele_bot
        .set_my_commands(Command::bot_commands())
        .await
        .expect("set command error");

    tokio::spawn(async move {
        Dispatcher::builder(
            tele_bot,
            Update::filter_message().branch(
                dptree::entry()
                    .filter_command::<Command>()
                    .endpoint(handler),
            ),
        )
        .dependencies(dptree::deps![ggs_client])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    });

    wait_for_signal().await;
}

// wait_for_signal: https://blog.logrocket.com/guide-signal-handling-rust/
async fn wait_for_signal() {
    let mut sigint = signal(SignalKind::interrupt()).expect("init signal interrupt error");

    match sigint.recv().await {
        Some(()) => println!("SIGINT received"),
        None => println!("sth went wrong"),
    }
}

async fn handler(
    bot: Bot,
    ggs: gg_sheet::GgsClient,
    // me: teloxide::types::Me,
    msg: Message,
    cmd: Command,
) -> Result<(), teloxide::RequestError> {
    match cmd {
        Command::AddBalance(s) => {
            let b_op = match BalanceOperation::new(&s) {
                Ok(b_op) => b_op,
                Err(e) => {
                    bot.send_message(msg.chat.id, format!("parse balance operation error  {}", e))
                        .await?;
                    return Ok(());
                }
            };
            bot.send_message(msg.chat.id, format!("{:#?}", b_op))
                .await?;
            return Ok(());
        }
        _ => {
            bot.send_message(msg.chat.id, "invalid command").await?;
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
    fn new(s: &str) -> anyhow::Result<BalanceOperation> {
        let valid_users = vec![String::from("tpp"), String::from("pch")];
        let valid_ops = vec![String::from("+"), String::from("-")];
        let valid_units = vec![String::from("vnd"), String::from("pl")];
        let mut b_op = BalanceOperation::default();

        let data = s.split(' ').collect::<Vec<&str>>();
        if data.len() != 4 && data.len() != 7 {
            return Err(anyhow::format_err!("invalid length {}", data.len()));
        }

        let user = data[0].to_lowercase();
        let op_1 = data[1].to_lowercase();
        let amount_1 = data[2].to_lowercase();
        let unit_1 = data[3].to_lowercase();

        if !valid_users.contains(&user) {
            return Err(anyhow::format_err!("invalid user: [{}]", user));
        }
        b_op.user = user;

        if !valid_ops.contains(&op_1) {
            return Err(anyhow::format_err!("invalid op 1: [{}]", op_1));
        }
        if op_1.as_str() == "+" {
            b_op.sign_positive_1 = true;
        }

        b_op.amount_1 = Self::parse_amount(&amount_1)?;

        let unit_1 = unit_1.to_lowercase();
        if !valid_units.contains(&unit_1) {
            return Err(anyhow::format_err!("invalid unit 1: [{}]", unit_1));
        }
        b_op.unit_1 = unit_1;

        if data.len() != 7 {
            return Ok(b_op);
        }

        let op_2 = data[4].to_lowercase();
        let amount_2 = data[5].to_lowercase();
        let unit_2 = data[6].to_lowercase();

        if !valid_ops.contains(&op_2) {
            return Err(anyhow::format_err!("invalid op 2: [{}]", op_2));
        }
        if op_2.as_str() == "+" {
            b_op.sign_positive_2 = true;
        }

        b_op.amount_2 = Self::parse_amount(&amount_2)?;

        let unit_2 = unit_2.to_lowercase();
        if !valid_units.contains(&unit_2) {
            return Err(anyhow::format_err!("invalid unit 2: [{}]", unit_2));
        }
        b_op.unit_2 = unit_2;

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
