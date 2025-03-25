pub mod configuration;
mod error;

use std::{net::TcpStream, pin::Pin, sync::Arc};

use error::NinoverseApiError;
use http::{Request, Response, StatusCode};
use sqlx::{Pool, Postgres};

use crate::http_handler;

pub async fn forward_incoming_request<T>(
    request: &Request<T>,
    pool: &Arc<Pool<Postgres>>,
    stream: &mut TcpStream,
    uri_section_configuration: &UriSection<T>,
) -> Result<(), NinoverseApiError> {
    let mut request_uri_parts = http_handler::extract_uri_pieces_vector(request);
    Ok(uri_section_configuration
        .next(request, pool, stream, &mut request_uri_parts)
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

pub struct UriSection<'a, 'b, T> {
    #[allow(dead_code)]
    section_type: UriSectionType,
    execute_uri_section: Box<
        dyn Fn(
                &'b Request<T>,
                &'b Arc<Pool<Postgres>>,
                &'b mut TcpStream,
            ) -> Pin<Box<dyn Future<Output = Result<(), NinoverseApiError>> + Send +'b>>
            + Send
            + Sync
            + 'a,
    >,
    // fn(&Request<T>, &Arc<Pool<Postgres>>, &mut TcpStream) -> Result<(), NinoverseApiError>,
    next_section: Box<dyn Fn(&'b Vec<String>) -> Option<UriSection<'a, 'b, T>> + Send + Sync + 'a> ,
}

trait UriSectionFn<'a, 'b, T> {
    async fn next(
        &self,
        request: &'b Request<T>,
        pool: &'b Arc<Pool<Postgres>>,
        stream: &'b mut TcpStream,
        request_uri_parts: &'b mut Vec<String>,
    ) -> Result<(), NinoverseApiError>;
}

impl<'a, 'b, T> UriSectionFn<'a, 'b, T> for UriSection<'a, 'b, T> {
    async fn next(
        &self,
        request: &'b Request<T>,
        pool: &'b Arc<Pool<Postgres>>,
        stream: &'b mut TcpStream,
        request_uri_parts: &'b mut Vec<String>,
    ) -> Result<(), NinoverseApiError> {
        (self.execute_uri_section)(request, pool, stream).await?;
        // Store the result of the immutable borrow in a temporary variable
        let next_section = (&self.next_section)(request_uri_parts);
        if let Some(next_section) = next_section {
            if !request_uri_parts.is_empty() {
                let mut request_uri_parts = request_uri_parts.clone();
                request_uri_parts.clone().remove(0);
                Ok(Box::pin(next_section.next(request, pool, stream, &mut request_uri_parts)).await?)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

enum UriSectionType {
    Root,
    Branch,
    Leaf,
}
