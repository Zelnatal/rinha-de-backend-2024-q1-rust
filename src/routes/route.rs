use axum::{
    routing::{get, post},
    Router,
};

use crate::config::AppState;

use super::{statements::statements_get, transactions::transaction_post};

pub fn get_all_routes() -> Router<AppState> {
    let group = Router::new()
        .route("/extrato", get(statements_get))
        .route("/transacoes", post(transaction_post));

    Router::new().nest("/clientes/:id", group)
}
