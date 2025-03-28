mod error;

use std::sync::Arc;

use actix_web::{App, HttpResponse, HttpServer, Responder, dev::Server, get, post, web};
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::Sender;

use crate::{KafkaChannelMessage, configuration_handler};

pub fn init_request_handler(
    pool: Arc<Pool<Postgres>>,
    kafka_thread_sender: Sender<KafkaChannelMessage>,
) -> Result<Server, Box<dyn std::error::Error>> {
    Ok(HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(kafka_thread_sender.clone()))
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .disable_signals()
    .bind(("0.0.0.0", configuration_handler::get_self_port().parse::<u16>()?))?
    .run())
}

#[get("/")]
async fn hello(
    kafka_thread_sender: web::Data<tokio::sync::mpsc::Sender<KafkaChannelMessage>>,
) -> impl Responder {
    let kafka_write_result = kafka_thread_sender
        .send(KafkaChannelMessage::Message {
            sender: String::from("base_endpoint"),
            content: String::from("Received a message!"),
        })
        .await;
    if let Err(kafka_write_error) = kafka_write_result {
        println!("{}", kafka_write_error);
    }
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

pub async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
