#[derive(Clone)]
pub struct Client {
    c: reqwest::Client,
    auth: String,
    org: String,
}

impl Client {
    pub fn new(auth: String, org: String) -> Client {
        Client {
            c: reqwest::Client::new(),
            auth,
            org,
        }
    }

    pub async fn list_devices(&self) -> anyhow::Result<ListDeviceResponse> {
        let url = format!(
            "https://api.tailscale.com/api/v2/tailnet/{}/devices",
            self.org
        );
        let pwd: Option<String> = None;
        let resp: ListDeviceResponse = self
            .c
            .get(url)
            .basic_auth(&self.auth, pwd)
            .send()
            .await?
            .json()
            .await?;

        Ok(resp)
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct ListDeviceResponse {
    pub devices: Vec<DeviceInfo>,
}

#[derive(serde::Deserialize, Debug)]
pub struct DeviceInfo {
    pub addresses: Vec<String>,
    pub id: String,
    #[serde(rename(deserialize = "nodeId"))]
    pub node_id: String,
    pub user: String,
    pub hostname: String,
    #[serde(rename(deserialize = "clientVersion"))]
    pub client_version: String,
    #[serde(rename(deserialize = "updateAvailable"))]
    pub update_available: bool,
    pub os: String,
    pub created: String,
    #[serde(rename(deserialize = "lastSeen"))]
    pub last_seen: String,
    #[serde(rename(deserialize = "keyExpiryDisabled"))]
    pub key_expiry_disabled: bool,
    pub expires: String,
    pub authorized: bool,
    #[serde(rename(deserialize = "isExternal"))]
    pub is_external: bool,
    #[serde(rename(deserialize = "machineKey"))]
    pub machine_key: String,
    #[serde(rename(deserialize = "nodeKey"))]
    pub node_key: String,
    #[serde(rename(deserialize = "blocksIncomingConnections"))]
    pub blocks_incoming_connections: bool,
}
