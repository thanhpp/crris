#[cfg(test)]
mod tests {
    use crate::{binanceclient::client::BinanceClient, *};
    use std::env;

    #[tokio::test]
    async fn test_get_open_orders() {
        let b_client = BinanceClient::new(
            env::var("KYB_DEV_BINANCE_READ_API_KEY").unwrap(),
            env::var("KYB_DEV_BINANCE_READ_SECRET_KEY").unwrap(),
        );

        let open_orders = b_client.get_open_order_service().exec().await.unwrap();

        tokio::io::stdout()
            .write(format!("test_get_open_orders {:?}\n", open_orders).as_bytes())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_account_info() {
        let b_client = BinanceClient::new(
            env::var("KYB_DEV_BINANCE_READ_API_KEY").unwrap(),
            env::var("KYB_DEV_BINANCE_READ_SECRET_KEY").unwrap(),
        );

        let open_orders = b_client.get_account_info_service().exec().await.unwrap();

        tokio::io::stdout()
            .write(format!("test_get_account_info {:?}\n", open_orders).as_bytes())
            .await
            .unwrap();
    }
}
