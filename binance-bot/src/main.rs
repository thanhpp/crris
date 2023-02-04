use binance_bot::BinanceClient;

#[tokio::main]
async fn main() {
    let c = BinanceClient::new("".into(), "".into());

    c.get_open_orders().await.unwrap();
}
