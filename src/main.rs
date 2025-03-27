mod api_handler;
mod configuration_handler;
mod db_handler;
mod http_handler;
mod kafka_handler;
mod logger;

// use logger::{log, LogLevel};

use std::{
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use actix_web::{App, HttpServer, dev::Server, web};
use api_handler::{echo, hello};
use http::Request;

use kafka_handler::init_kafka;

use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::Sender;

pub struct KafkaChannelMessage {
    sender: String,
    content: String
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("MAIN: Program started.");
    println!("MAIN: Testing configuration.");
    configuration_handler::test_configuration()?;
    println!("MAIN: Configuration loaded.");
    println!("MAIN: Initializing DB pool.");
    let pool = db_handler::init_db().await?;
    println!("MAIN: Starting threads");
    run_threads(Arc::new(pool)).await?;
    Ok(())

    // ThreadPool
}

async fn run_threads(pool: Arc<Pool<Postgres>>) -> Result<(), Box<dyn std::error::Error>> {
    let pool_tcp_clone = pool.clone();
    let (kafka_thread_sender, kafka_thread_receiver) = tokio::sync::mpsc::channel(100);
    let api_listener_thread_handler = tokio::spawn(async move {
        println!("RUN_THREADS: Starting API listener thread.");
        let _ = init_request_handler(pool_tcp_clone, kafka_thread_sender)
            .expect("Error in the HTTP Server.")
            .await;
    });
    let kafka_thread_handler = tokio::spawn(async {
        println!("RUN_THREADS: Starting KAFKA thread.");
        init_kafka(kafka_thread_receiver).await;
    });
    tokio::try_join!(api_listener_thread_handler, kafka_thread_handler)
        .expect("RUN_THREADS: Some error occured in the thread.");
    Ok(())
}

fn init_request_handler(
    pool: Arc<Pool<Postgres>>,
    kafka_thread_sender: Sender<KafkaChannelMessage>,
) -> Result<Server, Box<dyn std::error::Error>> {
    Ok(HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(kafka_thread_sender.clone()))
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(api_handler::manual_hello))
    })
    .disable_signals()
    .bind(("127.0.0.1", 7878))?
    .run())
}
