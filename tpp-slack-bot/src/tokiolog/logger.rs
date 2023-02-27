use chrono::Utc;
use tokio::io::AsyncWriteExt;

pub async fn log_info(message: String) {
    tokio::io::stdout()
        .write(format!("{:#?} {}\n", Utc::now().timestamp(), message).as_bytes())
        .await
        .unwrap();
}
