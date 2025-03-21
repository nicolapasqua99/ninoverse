use std::{net::TcpStream, sync::Arc};

use http::{Request, Response, StatusCode};
use sqlx::{Pool, Postgres};

use crate::{http_handler, TestFoo};

pub fn forward_incoming_request<T>(request: Request<T>, pool: &Arc<Pool<Postgres>>, stream: TcpStream) {
    match request.uri().path() {
        "/foo" => handle_generic_request::<T>(pool, stream),
        _ => handle_404_request(stream)
    }
}

fn handle_generic_request<T>(_pool: &Arc<Pool<Postgres>>, mut stream: TcpStream) {
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(TestFoo::default());
    if let Ok(response) = response {
        http_handler::write_to_stream(&mut stream, response);
    }
}

fn handle_404_request(mut stream: TcpStream) {
    http_handler::write_to_stream(&mut stream, Response::builder().status(StatusCode::NOT_FOUND).body(()).expect("REQUEST_HANDLER_404: Error while handling a 404 response"));
}
