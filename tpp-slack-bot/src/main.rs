use std::env;

use serde::Serialize;
mod slackclient;

#[tokio::main]
async fn main() {
    let slack_token = env::var("TPP_SLACK_API_TOKEN").expect("get slack token error");
    println!("GOT SLACK TOKEN FROM ENV");

    let req = SlackSendMessageReq {
        channel: String::from("C04N96G28F9"),
        text: String::from("hello, world"),
    };
    let req_serialized = serde_json::to_string(&req).unwrap();
    println!("req_serialized \n{}", &req_serialized);
    // write.send(req_serialized.into()).await.unwrap();

    let post_msg_txt = reqwest::Client::new()
        .post("https://slack.com/api/chat.postMessage")
        .header("Content-type", "application/json; charset=utf-8")
        .header("Authorization", format!("Bearer {}", &slack_token))
        .body(req_serialized)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("{}", post_msg_txt);
}

#[derive(Serialize)]
pub struct SlackSendMessageReq {
    pub channel: String,
    pub text: String,
}
