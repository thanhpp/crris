use actix_web::{get, web, App, Error, HttpResponse, HttpServer};

mod db;
mod dto;
mod schema;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(get)
            .service(web::resource("/").route(web::post().to(post_message)))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn post_message(msg: web::Json<dto::Message>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(dto::Message {
            message: msg.message.clone(),
            username: msg.username.clone(),
        }))
}

#[get("/")]
async fn get() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("hello, world"))
}
