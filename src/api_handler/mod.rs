pub mod configuration;
mod structs;
mod error;

pub use structs::*;

use std::{net::TcpStream, sync::Arc};

use error::NinoverseApiError;
use http::{Request, Response, StatusCode};
use sqlx::{Pool, Postgres};

use crate::http_handler;

pub async fn forward_incoming_request<T>(
    request: Request<T>,
    stream: TcpStream,
    pool: &Arc<Pool<Postgres>>,
    uri_section_configuration: &UriSection<T>,
) -> Result<(), NinoverseApiError> {
    let request_uri_parts = http_handler::extract_uri_pieces_vector(&request);
    Ok(uri_section_configuration
        .next(request, stream, pool, request_uri_parts)
        .await?)
}

async fn handle_404_request(mut stream: &mut TcpStream) -> Result<(), NinoverseApiError> {
    Ok(http_handler::write_to_stream(
        &mut stream,
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("No endpoint available.")
            .or_else(|error| {
                return Err(NinoverseApiError::GenericError {
                    original_error: format!("{}", error),
                });
            })?,
    )
    .or_else(|error| {
        Err(NinoverseApiError::GenericError {
            original_error: format!("{}", error),
        })
    })?)
}
