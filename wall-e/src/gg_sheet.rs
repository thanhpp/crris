// authenticate with google API using service account

use anyhow::Ok;
use google_sheets4::{api::ValueRange, oauth2::authenticator::Authenticator};
use serde_json as json;

#[derive(Clone)]
pub struct GgsClient {
    hub: google_sheets4::Sheets<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    sheet_id: String,
}

impl GgsClient {
    pub async fn new(private_key_file: &str, sheet_id: &str) -> anyhow::Result<GgsClient> {
        let http_client: hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>> =
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_only()
                    .enable_http1()
                    .build(),
            );

        let auth = Self::gen_auth(private_key_file, http_client.clone())
            .await
            .expect("gen auth error");

        let gg_hub = google_sheets4::Sheets::new(http_client.clone(), auth);

        let c = GgsClient {
            hub: gg_hub,
            sheet_id: String::from(sheet_id),
        };

        Ok(c)
    }

    pub async fn read_range(&self, range: &str) -> anyhow::Result<ValueRange> {
        let result = self
            .hub
            .spreadsheets()
            .values_get(&self.sheet_id, range)
            .doit()
            .await?;

        Ok(result.1)
    }

    pub async fn find_empty_row(&self, range: &str) -> anyhow::Result<i32> {
        let result = self
            .hub
            .spreadsheets()
            .values_get(&self.sheet_id, range)
            .doit()
            .await?;

        let vals = match result.1.values {
            None => return Ok(0),
            Some(v) => v,
        };

        let mut row: i32 = 1;
        for v in vals {
            if v.is_empty() {
                break;
            }

            match v.get(0) {
                None => break,
                Some(v) => match v.as_str() {
                    None => break,
                    Some(s) => {
                        if !s.is_empty() {
                            row += 1;
                        }
                    }
                },
            }
        }

        Ok(row)
    }

    pub async fn append_rows(&self, range: &str, data: Vec<&str>) -> anyhow::Result<()> {
        let mut values: Vec<json::Value> = Vec::new();
        for d in data.iter() {
            let val = json::to_value(d)?;
            values.push(val);
        }

        let _ = self
            .hub
            .spreadsheets()
            .values_append(
                ValueRange {
                    major_dimension: Some("ROWS".to_string()),
                    range: Some(String::from(range)),
                    values: Some(vec![values]),
                },
                &self.sheet_id,
                range,
            )
            .value_input_option("USER_ENTERED")
            .doit()
            .await?;

        Ok(())
    }

    async fn gen_auth(
        priv_key_file: &str,
        client: hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    ) -> anyhow::Result<Authenticator<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>>
    {
        let secret: google_sheets4::oauth2::ServiceAccountKey =
            google_sheets4::oauth2::read_service_account_key(priv_key_file).await?;

        let auth = google_sheets4::oauth2::ServiceAccountAuthenticator::with_client(secret, client)
            .build()
            .await?;

        Ok(auth)
    }
}
