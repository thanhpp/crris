use crate::*;

pub struct Service<'a> {
    // input
    env: String,
    epsilon: f64,

    // data
    current_balance: HashMap<String, f64>,
    current_balance_update: chrono::DateTime<chrono::Utc>,
    previous_balance: HashMap<String, f64>,
    previous_balance_update: chrono::DateTime<chrono::Utc>,

    // clients
    cex_dex: cexdexclient::client::CexDexClient,
    slack: &'a slackclient::client::Client,
}

impl<'a> Service<'a> {
    pub fn new(
        env: String,
        epsilon: f64,
        cex_dex: cexdexclient::client::CexDexClient,
        slack: &'a slackclient::client::Client,
    ) -> Service {
        Self {
            env,
            epsilon,
            current_balance: HashMap::new(),
            current_balance_update: chrono::Utc::now(),
            previous_balance: HashMap::new(),
            previous_balance_update: chrono::Utc::now(),
            cex_dex,
            slack,
        }
    }

    pub async fn monitor_balance(&mut self) {
        let fetch_limit = 1;
        loop {
            thread::sleep(Duration::from_secs(5));

            let mut balance = HashMap::<String, f64>::new();
            let mut fetch_count = fetch_limit;
            while fetch_count > 0 {
                // if the balance is not changed in 1 minutes, consider it is the true balance
                thread::sleep(Duration::from_secs(10));
                println!("fetching balance..., count: {}", fetch_count);
                match self.fetch_balance().await {
                    Err(e) => {
                        println!("fetch balance error {}", e);
                        fetch_count = fetch_limit;
                    }
                    Ok(b) => {
                        if balance.is_empty() {
                            balance = b;
                            fetch_count -= 1;
                            continue;
                        }
                        if !self.calculate_diff(&balance, &b).is_empty() {
                            balance = b;
                            fetch_count = fetch_limit;
                            continue;
                        }
                        balance = b;
                        fetch_count -= 1;
                    }
                }
            }

            let now = chrono::Utc::now();

            if self.current_balance.is_empty() || self.previous_balance.is_empty() {
                self.current_balance = balance.clone();
                self.current_balance_update = now;
                self.previous_balance = balance;
                self.previous_balance_update = now;

                if let Err(e) = self.send_balances_msg(&self.current_balance).await {
                    println!("send empty balance error {}", e);
                }
                println!("sent empty balance");
                continue;
            }

            if now - self.current_balance_update >= chrono::Duration::hours(1) {
                let diff = self.calculate_diff(&self.current_balance, &balance);
                if !diff.is_empty() {
                    if let Err(e) = self
                        .send_diff_msg(&self.current_balance_update, &now, &diff)
                        .await
                    {
                        println!("send diff 1h error {}", e);
                    }
                    println!("sent 1h diff")
                }
                self.current_balance = balance.clone();
                self.current_balance_update = now;
            }

            if now - self.previous_balance_update >= chrono::Duration::hours(24) {
                let diff = self.calculate_diff(&self.previous_balance, &balance);
                if !diff.is_empty() {
                    if let Err(e) = self
                        .send_diff_msg(&self.previous_balance_update, &now, &diff)
                        .await
                    {
                        println!("send diff 24h error {}", e);
                    }
                    println!("sent 24h diff");

                    if let Err(e) = self.send_balances_msg(&balance).await {
                        println!("send balance 24h error {}", e)
                    }
                    println!("sent 24h balance")
                }
                self.previous_balance = balance;
                self.previous_balance_update = now;
            }

            thread::sleep(Duration::from_secs(600));
        }
    }

    async fn fetch_balance(&self) -> anyhow::Result<HashMap<String, f64>> {
        let cex_balance = self.cex_dex.get_cex_balanace().await?;
        if cex_balance.data.is_rebalancing {
            return Err(anyhow::format_err!("cex balance is rebalancing"));
        }

        let dex_balance = self.cex_dex.get_dex_balanace().await?;
        if dex_balance.data.is_rebalancing {
            return Err(anyhow::format_err!("dex balance is rebalancing"));
        }

        let mut balance = HashMap::<String, f64>::new();

        for (k, v) in cex_balance.data.balances {
            match balance.get_mut(&k) {
                None => {
                    balance.insert(k, v.free + v.locked);
                }
                Some(b) => *b += v.free + v.locked,
            }
        }

        for (k, v) in dex_balance.data.contract_balances {
            match balance.get_mut(&k) {
                None => {
                    balance.insert(k, v);
                }
                Some(b) => *b += v,
            }
        }

        for (k, v) in dex_balance.data.balances {
            match balance.get_mut(&k) {
                None => {
                    balance.insert(k, v);
                }
                Some(b) => *b += v,
            }
        }

        Ok(balance)
    }

    fn calculate_diff(
        &self,
        last_balances: &HashMap<String, f64>,
        curr_balances: &HashMap<String, f64>,
    ) -> Vec<(String, f64)> {
        let mut diff_map = curr_balances.clone();

        for (k, v) in last_balances.iter() {
            match diff_map.get_mut(k) {
                None => {
                    diff_map.insert(k.clone(), -*v);
                }
                Some(b) => *b -= *v,
            }
        }

        let mut diff_vec = diff_map
            .drain()
            .filter(|(_, v)| v.abs() >= self.epsilon)
            .map(|(k, v)| (k, v))
            .collect::<Vec<(String, f64)>>();
        diff_vec.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        diff_vec
    }

    async fn send_diff_msg(
        &self,
        last_balance_update: &chrono::DateTime<chrono::Utc>,
        utc_now: &chrono::DateTime<chrono::Utc>,
        diff_vec: &[(String, f64)],
    ) -> anyhow::Result<()> {
        let mut msg = format!(
            "*****
    *ASSET DIFF*
    > ENV: {}
    > {} -> {}
    ",
            self.env,
            last_balance_update.to_rfc3339(),
            utc_now.to_rfc3339()
        );

        for (asset, diff) in diff_vec.iter() {
            if *diff == 0.0 {
                continue;
            }
            msg.push_str(format!("{}: {}\n", asset, diff).as_str())
        }

        match self
            .slack
            .send_message(String::from("alert-virtual-taker-1"), msg)
            .await
        {
            Ok(()) => Ok(()),
            Err(e) => Err(anyhow::format_err!("{}", e)),
        }
    }

    async fn send_balances_msg(&self, balances: &HashMap<String, f64>) -> anyhow::Result<()> {
        let mut balances_vec: Vec<(String, f64)> =
            balances.iter().map(|(k, v)| (k.clone(), *v)).collect();

        balances_vec.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut msg = format!(
            "******
*BALANCES*
> ENV: {}
",
            self.env,
        );

        for (asset, diff) in balances_vec.iter() {
            msg.push_str(format!("{}: {}\n", asset, diff).as_str());
        }

        match self
            .slack
            .send_message(String::from("alert-virtual-taker-1"), msg)
            .await
        {
            Ok(()) => Ok(()),
            Err(e) => Err(anyhow::format_err!("{}", e)),
        }
    }
}
