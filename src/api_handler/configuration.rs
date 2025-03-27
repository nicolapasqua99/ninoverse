use std::{pin::Pin, sync::Arc};

use sqlx::{Executor, Pool, Postgres};

use super::{UriSection, error::NinoverseApiError};

pub fn get_configuration<T>() -> UriSection<T> {
    UriSection {
        execute_uri_section: |_request, _stream, _pool, request_uri_parts| {
            let uri_part_str;
            if let Some(uri_part) = request_uri_parts.get(0) {
                uri_part_str = uri_part.as_str();
            } else {
                uri_part_str = "_";
            }
            match uri_part_str {
                "project" => Box::pin(async { Ok(Some(get_project_handler())) }),
                _ => Box::pin(async { Ok(None) }),
            }
        },
    }
}

fn get_404_handler<T>() -> UriSection<T> {
    UriSection {
        execute_uri_section: |_request, _stream, _pool, request_uri_parts| {
            let uri_part_str;
            if let Some(uri_part) = request_uri_parts.get(0) {
                uri_part_str = uri_part.as_str();
            } else {
                uri_part_str = "_";
            }
            match uri_part_str {
                _ => Box::pin(async { Ok(None) }),
            }
        },
    }
}

fn get_project_handler<T>() -> UriSection<T> {
    UriSection {
        execute_uri_section: |_request, _stream, _pool, request_uri_parts| {
            let uri_part_str;
            if let Some(uri_part) = request_uri_parts.get(0) {
                uri_part_str = uri_part.as_str();
            } else {
                uri_part_str = "_";
            }
            match uri_part_str {
                "add" => Box::pin(async { Ok(Some(get_add_project_handler())) }),
                _ => Box::pin(async { Ok(None) }),
            }
        },
    }
}

fn get_add_project_handler<T>() -> UriSection<T> {
    UriSection {
        execute_uri_section: Box::pin(
            async |_request, _stream, pool, _request_uri_parts: &Vec<String>| {
                add_project(pool).await;
                Ok(None)
            },
        ),
    }
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
