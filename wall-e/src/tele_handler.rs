use chrono::Datelike;
use teloxide::{
    dispatching::UpdateFilterExt, prelude::*, types::Message, utils::command::BotCommands, Bot,
};

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

pub struct TeleHandler {}

impl TeleHandler {
    pub async fn start(
        cfg: crate::config::MainConfig,
        ggs_client: crate::gg_sheet::GgsClient,
    ) -> anyhow::Result<()> {
        let tele_bot = Bot::new(cfg.telegram_token.clone());
        tele_bot.set_my_commands(Command::bot_commands()).await?;

        tokio::spawn(async move {
            Dispatcher::builder(
                tele_bot,
                Update::filter_message().branch(
                    dptree::entry()
                        .filter_command::<Command>()
                        .endpoint(Self::handler),
                ),
            )
            .dependencies(dptree::deps![ggs_client, cfg])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
        });

        Ok(())
    }

    async fn handler(
        bot: Bot,
        ggs: crate::gg_sheet::GgsClient,
        cfg: crate::config::MainConfig,
        // me: teloxide::types::Me,
        msg: Message,
        cmd: Command,
    ) -> Result<(), teloxide::RequestError> {
        if msg.chat.id.to_string().ne(&cfg.add_balance_config.chat_id) {
            bot.send_message(msg.chat.id, "forbidden chat").await?;
            return Ok(());
        }

        match cmd {
            Command::AddBalance(s) => {
                let b_op = match BalanceOperation::new(&s) {
                    Ok(b_op) => b_op,
                    Err(e) => {
                        bot.send_message(
                            msg.chat.id,
                            format!("parse balance operation error  {}", e),
                        )
                        .await?;
                        return Ok(());
                    }
                };

                if let Err(e) =
                    write_balance_op(&ggs, &b_op, &cfg.add_balance_config.write_range).await
                {
                    bot.send_message(msg.chat.id, format!("{} {:#?}", e, &b_op))
                        .await?;
                    return Ok(());
                }
                match Self::send_balance(&ggs, &cfg.add_balance_config.balance_range, &bot, &msg)
                    .await
                {
                    Ok(_) => return Ok(()),
                    Err(e) => return Err(e),
                };
            }
            Command::ShowBalance => {
                match Self::send_balance(&ggs, &cfg.add_balance_config.balance_range, &bot, &msg)
                    .await
                {
                    Ok(_) => return Ok(()),
                    Err(e) => return Err(e),
                };
            }
            _ => {
                bot.send_message(msg.chat.id, "invalid command").await?;
            }
        };

        Ok(())
    }

    async fn send_balance(
        ggs: &crate::gg_sheet::GgsClient,
        balance_range: &str,
        b: &Bot,
        msg: &Message,
    ) -> anyhow::Result<(), teloxide::RequestError> {
        let balances = match get_current_balance(ggs, balance_range).await {
            Err(e) => {
                b.send_message(msg.chat.id, format!("get current balance error {}", e))
                    .await?;
                return Ok(());
            }
            Ok(b) => b,
        };

        let mut response = String::from("---tpp---\n");
        for (i, b) in balances.iter().enumerate() {
            if i == 2 {
                response.push_str("---pch---\n");
            }
            response.push_str(b);
            response.push('\n');
        }

        b.send_message(msg.chat.id, response).await?;

        Ok(())
    }
}

async fn write_balance_op(
    ggs: &crate::gg_sheet::GgsClient,
    b_op: &BalanceOperation,
    write_range: &str,
) -> anyhow::Result<()> {
    // range A-E
    ggs.append_rows(
        write_range,
        b_op.to_values().iter().map(|s| &**s).collect::<Vec<&str>>(),
    )
    .await?;

    Ok(())
}

async fn get_current_balance(
    ggs: &crate::gg_sheet::GgsClient,
    read_range: &str,
) -> anyhow::Result<Vec<String>> {
    let values = ggs.read_range(read_range).await?;

    let mut res = Vec::<String>::new();

    let values = match values.values {
        None => return Ok(res),
        Some(vals) => vals,
    };

    for v in values.iter() {
        for val in v.iter() {
            res.push(val.to_string())
        }
    }

    Ok(res)
}

fn get_date() -> String {
    let n = chrono::Local::now();
    format!("{}/{}/{}", n.day(), n.month(), n.year())
}

#[derive(Default, Debug)]
struct BalanceOperation {
    user: String,

    amount_1: f64,
    unit_1: String,

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
        let mut is_pos = false;
        if op_1.as_str() == "+" {
            is_pos = true;
        }
        b_op.amount_1 = Self::parse_amount(&amount_1)?;
        if !is_pos {
            b_op.amount_1 *= -1.0;
        }

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
        let mut is_pos = false;
        if op_2.as_str() == "+" {
            is_pos = true;
        }

        b_op.amount_2 = Self::parse_amount(&amount_2)?;
        if !is_pos {
            b_op.amount_2 *= -1.0;
        }

        let unit_2 = unit_2.to_lowercase();
        if !valid_units.contains(&unit_2) {
            return Err(anyhow::format_err!("invalid unit 2: [{}]", unit_2));
        }
        b_op.unit_2 = unit_2;

        if b_op.unit_1 == b_op.unit_2 {
            return Err(anyhow::format_err!("identical unit [{}]", b_op.unit_1));
        }

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

    fn to_values(&self) -> Vec<String> {
        // 1 - date
        // 2 - tpp VND
        // 3 - tpp PL
        // 4 - pch VND
        // 5 - pch PL

        let mut v: Vec<String> = vec![get_date()];
        if self.user == "tpp" {
            if self.unit_1 == "vnd" {
                v.push(self.amount_1.to_string().replace('.', ","));
                v.push(self.amount_2.to_string().replace('.', ","));
            } else {
                v.push(self.amount_2.to_string().replace('.', ","));
                v.push(self.amount_1.to_string().replace('.', ","));
            }
            v.push(0.to_string());
            v.push(0.to_string());
        } else {
            v.push(0.to_string());
            v.push(0.to_string());
            if self.unit_1 == "vnd" {
                v.push(self.amount_1.to_string().replace('.', ","));
                v.push(self.amount_2.to_string().replace('.', ","));
            } else {
                v.push(self.amount_2.to_string().replace('.', ","));
                v.push(self.amount_1.to_string().replace('.', ","));
            }
        }

        v
    }
}
