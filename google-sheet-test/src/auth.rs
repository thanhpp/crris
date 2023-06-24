// authenticate with google API using service account

use google_sheets4::oauth2::authenticator::Authenticator;

pub async fn gen_auth(
    priv_key_file: &str,
    client: hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
) -> anyhow::Result<Authenticator<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>> {
    let secret: google_sheets4::oauth2::ServiceAccountKey =
        google_sheets4::oauth2::read_service_account_key(priv_key_file).await?;

    let auth = google_sheets4::oauth2::ServiceAccountAuthenticator::with_client(secret, client)
        .build()
        .await?;

    Ok(auth)
}
