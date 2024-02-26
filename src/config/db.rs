use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub async fn create_pool() -> Pool<Postgres> {
    PgPoolOptions::new()
        .max_connections(165)
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL não definida"))
        .await
        .expect("Erro no pool")
}
