use std::{net::TcpStream, pin::Pin, sync::Arc};

use http::Request;
use sqlx::{Pool, Postgres};

use super::error::NinoverseApiError;

pub struct UriSection<'a, T>
where
    T: Sync,
{
    pub execute_uri_section: Box<
        dyn Fn(
                &'a Request<T>,
                &'a TcpStream,
                Arc<Pool<Postgres>>,
            ) -> Pin<Box<dyn Future<Output = Result<(), NinoverseApiError>> + Send>>
            + Send
            + Sync
            +'a,
    >,
    pub next_section: Option<Box<UriSection<'a, T>>>,
}

pub trait UriSectionFn<'a, T>
where
    T: Sync,
{
    async fn next(
        &self,
        request: &'a Request<T>,
        stream: &'a TcpStream,
        pool: Arc<Pool<Postgres>>,
    ) -> Result<(), NinoverseApiError>;
}

impl<'a, T> UriSectionFn<'a, T> for UriSection<'a, T>
where
    T: Sync,
{
    async fn next(
        &self,
        request: &'a Request<T>,
        stream: &'a TcpStream,
        pool: Arc<Pool<Postgres>>,
    ) -> Result<(), NinoverseApiError> {
        let pool_clone = pool.clone();
        (self.execute_uri_section)(request, stream, pool).await?;
        if let Some(next_section) = &self.next_section {
            Box::pin(next_section.next(request, stream, pool_clone)).await?;
            Ok(())
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




use std::{net::TcpStream, pin::Pin, sync::Arc};

use http::Request;
use sqlx::{Executor, Pool, Postgres};

use super::{UriSection, error::NinoverseApiError};

async fn test<T>(
    request: &Request<T>,
    _stream: &TcpStream,
    _pool: &Arc<Pool<Postgres>>,
) -> Result<(), NinoverseApiError>
where
    T: Sync,
{
    println!("Executing URI section logic...");
    Ok(())
}

pub fn get_configuration<T>(request_uri_parts: Vec<String>) -> UriSection<T>
where
    T: Sync,
{
    UriSection {
        execute_uri_section: Box::new(|request, stream, pool| {
            Box::pin(async move { test(request, stream, pool).await })
        }),
        next_section: {
            let uri_part_str;
            if let Some(uri_part) = request_uri_parts.get(0) {
                uri_part_str = uri_part.as_str();
            } else {
                uri_part_str = "_";
            }
            match uri_part_str {
                "project" => Some(Box::new(get_project_handler())),
                _ => None,
            }
        },
    }
}

fn get_project_handler<T>() -> UriSection<T>
where
    T: Sync,
{
}

async fn add_project(pool: &Arc<Pool<Postgres>>) {
    pool.execute(
        "INSERT INTO projects (name, description) VALUES ('New Project', 'Project Description')",
    )
    .await;
}

// let uri_part_str;
// if let Some(uri_part) = request_uri_parts.get(0) {
//     uri_part_str = uri_part.as_str();
// } else {
//     uri_part_str = "_";
// }
// match uri_part_str {
//     "add" => Ok(Some(get_add_project_handler())),
//     _ =>  Ok(None),
// };
