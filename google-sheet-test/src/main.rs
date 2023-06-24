mod auth;
mod config;

const CONFIG_FILE: &str = "config.json";

#[tokio::main]
async fn main() {
    // authentication
    let cfg = config::Config::new(CONFIG_FILE).expect("read config file error");
    let http_client: hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>> =
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_only()
                .enable_http1()
                .build(),
        );
    let auth = auth::gen_auth(&cfg.priv_key_file, http_client.clone())
        .await
        .expect("gen auth error");

    // create google sheet controller (hub)
    let ggs_hub = google_sheets4::Sheets::new(http_client.clone(), auth);

    // read from configured sheets & range
    let result = ggs_hub
        .spreadsheets()
        .values_get(&cfg.sheet_id, &cfg.read_range)
        .doit()
        .await
        .expect("read google sheet error");

    println!("read result {:#?}", result);
}
