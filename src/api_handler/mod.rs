mod error;
mod projects;

use std::sync::Arc;

use actix_web::{HttpResponse, Responder, get, post, web};
use sqlx::{Pool, Postgres};

use crate::KafkaChannelMessage;

#[get("/")]
async fn hello(
    pool: web::Data<Arc<Pool<Postgres>>>,
    kafka_thread_sender: web::Data<tokio::sync::mpsc::Sender<KafkaChannelMessage>>,
) -> impl Responder {
    println!("{}", pool.num_idle());
    let _ = kafka_thread_sender.send(KafkaChannelMessage { sender: String::from("/"), content: String::from("Received a message!") }).await;
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

pub async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
