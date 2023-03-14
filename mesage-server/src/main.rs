use std::error::Error;

use actix_web::{get, web, App, HttpResponse, HttpServer};
use chrono::Utc;
use deadpool_diesel::postgres::{Manager, Pool};

mod db;
mod dto;
mod schema;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // create connection pool
    let manager = Manager::new(
        "postgres://username:password@localhost:5432/demo",
        deadpool_diesel::Runtime::Tokio1,
    );
    let pool = Pool::builder(manager).max_size(8).build().unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(get)
            .service(web::resource("/").route(web::post().to(post_message)))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn post_message(
    msg: web::Json<dto::Message>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => return Err(e.into()),
    };

    let db_msg = db::Message {
        timestamp: Utc::now().timestamp_nanos(),
        message: (&msg).message.clone(),
        username: (&msg).username.clone(),
    };
    let insert_db_msg = db_msg.clone();

    match conn
        .interact(move |conn| db::create_message(conn, &insert_db_msg))
        .await
    {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type("application/json")
            .json(db_msg)),
        Err(e) => Err(e.into()),
    }
}

#[get("/")]
async fn get() -> Result<HttpResponse, Box<dyn Error>> {
    Ok(HttpResponse::Ok().body("hello, world"))
}
