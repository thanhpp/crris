#[cfg(test)]
mod tests {
    use crate::{binanceclient::client::BinanceClient, *};

    #[tokio::test]
    async fn test_get_open_orders() {
        let b_client = BinanceClient::new(
            env::var("KYB_DEV_BINANCE_READ_API_KEY").unwrap(),
            env::var("KYB_DEV_BINANCE_READ_SECRET_KEY").unwrap(),
        );

        let open_orders = b_client.GetOpenOrderService().Do().await;

        tokio::io::stdout()
            .write(format!("test_get_open_orders {}\n", open_orders).as_bytes())
            .await
            .unwrap();
    }
}
