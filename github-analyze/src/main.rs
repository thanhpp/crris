use hyper::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    let uri = "http://httpbin.org/ip".parse()?;

    let resp = client.get(uri).await?;

    println!("Response {}", resp.status());

    Ok(())
}
