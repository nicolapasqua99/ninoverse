use std::sync::Arc;

use sqlx::{Executor, Pool, Postgres};

use super::{UriSection, error::NinoverseApiError};

pub fn get_configuration<T>() -> UriSection<T> {
    UriSection {
        execute_uri_section: Box::new(|_request, _pool, _stream| Box::pin(async { Ok(()) })),
        next_section: Box::new(|request_uri_parts| {
            let uri_part_str;
            if let Some(uri_part) = request_uri_parts.get(0) {
                uri_part_str = uri_part.as_str();
            } else {
                uri_part_str = "_";
            }
            match uri_part_str {
                "project" => Some(get_project_handler()),
                _ => Some(get_404_handler()),
            }
        }),
    }
}

fn get_404_handler<T>() -> UriSection<T> {
    UriSection {
        execute_uri_section: Box::new(|_request, _pool, stream| {
            Box::pin(async { super::handle_404_request(stream).await })
        }),
        next_section: Box::new(|request_uri_parts| {
            let uri_part_str;
            if let Some(uri_part) = request_uri_parts.get(0) {
                uri_part_str = uri_part.as_str();
            } else {
                uri_part_str = "_";
            }
            match uri_part_str {
                _ => None,
            }
        }),
    }
}

fn get_project_handler<T>() -> UriSection<T> {
    UriSection {
        execute_uri_section: Box::new(|_request, _pool, _stream| Box::pin(async { Ok(()) })),
        next_section: Box::new(|request_uri_parts| {
            let uri_part_str;
            if let Some(uri_part) = request_uri_parts.get(0) {
                uri_part_str = uri_part.as_str();
            } else {
                uri_part_str = "_";
            }
            match uri_part_str {
                "add" => Some(get_add_project_handler()),
                _ => Some(get_404_handler()),
            }
        }),
    }
}

fn get_add_project_handler<T>() -> UriSection<T> {
    UriSection {
        execute_uri_section: Box::new(|_request, pool, _stream| {
            Box::pin(async { add_project(pool).await })
        }),
        next_section: Box::new(|request_uri_parts| {
            let uri_part_str;
            if let Some(uri_part) = request_uri_parts.get(0) {
                uri_part_str = uri_part.as_str();
            } else {
                uri_part_str = "_";
            }
            match uri_part_str {
                _ => Some(get_404_handler()),
            }
        }),
    }
}

async fn add_project(pool: &Arc<Pool<Postgres>>) -> Result<(), NinoverseApiError> {
    pool.execute(
        "INSERT INTO projects (name, description) VALUES ('New Project', 'Project Description')",
    )
    .await;
    Ok(())
}
