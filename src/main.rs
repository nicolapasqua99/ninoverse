mod api_handler;
mod configuration_handler;
mod db_handler;
// mod http_handler;
mod kafka_handler;
mod logger;

// use logger::{log, LogLevel};

use std::sync::Arc;

use api_handler::init_request_handler;

use kafka_handler::init_kafka;

use sqlx::{Pool, Postgres};

pub enum KafkaChannelMessage {
    KafkaProducerStarted,
    KafkaProducerError,
    KafkaConsumerStarted,
    KafkaConsumerError,
    Message {
        sender: String,
        content: String,
    }
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
    let kafka_thread_sender_tcp = kafka_thread_sender.clone();
    let api_listener_thread_handler = tokio::spawn(async move {
        println!("RUN_THREADS: Starting API listener thread.");
        let _ = init_request_handler(pool_tcp_clone, kafka_thread_sender_tcp)
            .expect("RUN_THREADS: Error in the HTTP Server.")
            .await;
    });
    let kafka_thread_handler = tokio::spawn(async move {
        println!("RUN_THREADS: Starting KAFKA thread.");
        init_kafka(kafka_thread_sender, kafka_thread_receiver).await;
    });
    tokio::try_join!(api_listener_thread_handler, kafka_thread_handler)
        .expect("RUN_THREADS: Some error occured in the thread.");
    Ok(())
}
