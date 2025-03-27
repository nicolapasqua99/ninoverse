mod migration;

use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use super::configuration_handler::{
    get_pg_db, get_pg_host, get_pg_password, get_pg_port, get_pg_user,
};

async fn get_pool() -> Result<Pool<Postgres>, sqlx::Error> {
    let connection_string = format!(
        "postgres://{}:{}@{}:{}/{}",
        get_pg_user(),
        get_pg_password(),
        get_pg_host(),
        get_pg_port(),
        get_pg_db()
    );
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(connection_string.as_str())
        .await?;
    Ok(pool)
}

async fn migrate_db(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    migration::run_migrations(&pool).await
}

pub async fn init_db() -> Result<Pool<Postgres>, sqlx::Error> {
    let mut pool: Result<Pool<Postgres>, sqlx::Error> = Err(sqlx::Error::PoolClosed);
    for _ in 0..5 {
        println!("DB_INIT: Trying to connect to the database...");
        pool = get_pool().await;
        if pool.is_ok() {
            println!("DB_INIT: Connected successfully to the database");
            break;
        }
    }
    if let Ok(pool) = pool {
        println!("DB_INIT: Running migrations");
        migrate_db(&pool)
            .await
            .expect("DB_INIT: Error while running migrations");
        println!("DB_INIT: Migrations run successfully");
        Ok(pool)
    } else {
        pool
    }
}
