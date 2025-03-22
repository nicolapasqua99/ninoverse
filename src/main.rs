mod api;
mod configuration;
mod db;
mod http_handler;
mod kafka_handler;
mod migration;

use std::{
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use http::Request;

use kafka_handler::init_kafka;

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    println!("MAIN: Starting program.");
    configuration::test_configuration().expect("Configuration error");
    println!("MAIN: Initialize DB");
    let pool = db::init_db().await.expect("Database error");
    let tcp_listener_thread_handler = tokio::spawn(async {
        println!("MAIN: Starting TCP listener thread.");
        init_listener(pool).expect("MAIN: Error occured in TCP_LISTENER.");
    });
    let kafka_thread_handler = tokio::spawn(async {
        println!("MAIN: Starting KAFKA thread.");
        init_kafka().await;
    });

    tokio::try_join!(tcp_listener_thread_handler, kafka_thread_handler).expect("MAIN: Some error occured in the thread.");
    Ok(())
}

fn init_listener(pool: Arc<Pool<Postgres>>) -> Result<(), Box<dyn std::error::Error>>{
    let listener =
        TcpListener::bind(format!("0.0.0.0:{}", configuration::get_self_port())).expect("TCP_LISTENER_INIT: Error while initializing the TCP Listener.");
    for stream in listener.incoming() {
        let stream = stream.expect("TCP_LISTENER: Error in stream while receiving a message.");
        // thread::spawn(|| {
        handle_client(stream, &pool)?;
        // });
    }
    Ok(())
}

#[derive(Deserialize, Debug, Serialize)]
struct TestZot {
    zot: u8,
}

#[derive(Deserialize, Debug, Serialize)]
struct TestBar {
    bar: TestZot,
}

#[derive(Deserialize, Debug, Serialize)]
struct TestFoo {
    foo: TestBar,
}

impl Default for TestFoo {
    fn default() -> Self {
        TestFoo {
            foo: TestBar {
                bar: TestZot { zot: 0 },
            },
        }
    }
}

fn handle_client(mut stream: TcpStream, pool: &Arc<Pool<Postgres>>) -> Result<(), Box<dyn std::error::Error>>{
    let request: Request<()> = http_handler::read_from_stream(&mut stream)?;
    println!("Extracted value: {:?}", request.body());
    api::forward_incoming_request(request, pool, stream)?;
    Ok(())
}
