use sqlx::{Pool, Postgres, migrate::Migrator};

pub async fn run_migrations(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    let migrator = Migrator::new(std::path::Path::new("./sql")).await?;
    migrator.run(pool).await?;
    Ok(())
}
