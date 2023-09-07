#[cfg(test)]
mod tests {
    use std::fs;

    use serde::Deserialize;

    use crate::slackclient::client::{Client, SlackClientConfig};

    #[tokio::test]
    async fn send_webhook() {
        let cfg = must_read_config();

        let mut client = Client::new();
        client.add_webhook("alert-test".to_string(), cfg);

        let resp = client
            .send_message(
                "alert-test".to_string(),
                "> testing\ncex-dex-monitor".to_string(),
            )
            .await;
        if let Err(e) = resp {
            panic!("send messager error {}", e)
        }
    }

    #[derive(Deserialize)]
    struct Config {
        slack_client_config: SlackClientConfig,
    }

    fn must_read_config() -> String {
        let contents = fs::read_to_string("secret.yaml").unwrap();

        let cfg: Config = serde_yaml::from_str(&contents).unwrap();

        match cfg.slack_client_config.webhooks {
            None => panic!("webhook not found"),
            Some(w) => {
                if w.is_empty() {
                    panic!("empty webhook");
                };
                w[0].webhook.to_string() // copy
            }
        }
    }
}
