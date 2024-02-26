use axum::Router;
use sqlx::{Pool, Postgres};

use crate::routes::get_all_routes;

use super::db::create_pool;

#[derive(Debug, Clone)]
pub struct AppState{
    pub pool: Pool<Postgres>
}

pub async fn create_app() -> Router {
    let state = AppState {
        pool: create_pool().await
    };
    Router::new().nest("/", get_all_routes()).with_state(state)
}