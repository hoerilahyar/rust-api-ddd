use sqlx::PgPool;
use sqlx::{raw_sql, AssertSqlSafe};
use std::path::Path;

/// Applies every pending migration under `databases/postgresql/migrations`.
/// Uses sqlx's built-in migrator (tracks applied versions in
/// `_sqlx_migrations`), embedded at compile time.
pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("databases/postgresql/migrations")
        .run(pool)
        .await?;

    tracing::info!("migrations applied");
    Ok(())
}

/// Runs the idempotent seed scripts under `databases/postgresql/seeders`
/// in filename order. Each seeder is plain SQL with `ON CONFLICT DO NOTHING`
/// guards, so this is safe to call every boot in non-production environments.
pub async fn run_seeders(pool: &PgPool, seeders_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    let dir = seeders_dir.as_ref();

    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "sql"))
        .collect();

    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let sql = std::fs::read_to_string(&path)?;

        tracing::info!(file = %path.display(), "applying seeder");

        raw_sql(AssertSqlSafe(sql.as_str())).execute(pool).await?;
    }

    tracing::info!("seeders applied");
    Ok(())
}
