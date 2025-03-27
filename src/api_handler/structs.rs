use std::{net::TcpStream, pin::Pin, sync::Arc};

use http::Request;
use sqlx::{Pool, Postgres};

use super::error::NinoverseApiError;

pub struct UriSection<T> {
    pub execute_uri_section: Pin<
        Box<
            dyn Fn(
                &Request<T>,
                &TcpStream,
                &Arc<Pool<Postgres>>,
                &Vec<String>,
            ) -> Box<
                dyn std::future::Future<Output = Result<Option<UriSection<T>>, NinoverseApiError>>,
            >,
        >,
    >,
}

pub trait UriSectionFn<T> {
    async fn next(
        &self,
        request: Request<T>,
        stream: TcpStream,
        pool: &Arc<Pool<Postgres>>,
        request_uri_parts: Vec<String>,
    ) -> Result<(), NinoverseApiError>;
}

impl<T> UriSectionFn<T> for UriSection<T> {
    async fn next(
        &self,
        request: Request<T>,
        stream: TcpStream,
        pool: &Arc<Pool<Postgres>>,
        mut request_uri_parts: Vec<String>,
    ) -> Result<(), NinoverseApiError> {
        if !request_uri_parts.is_empty() {
            request_uri_parts.remove(0);
            let next_section =
                (self.execute_uri_section)(&request, &stream, &pool, &request_uri_parts).await?;
            if let Some(next_section) = next_section {
                (next_section.next(request, stream, pool, request_uri_parts)).await?;
                Ok(())
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
