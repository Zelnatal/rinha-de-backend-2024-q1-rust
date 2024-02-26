use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{query_as, Error};

use crate::{
    config::AppState,
    models::{Customers, Transactions, TransactionsKindDb},
};

use super::common::{ResponseError, TransactionKind};

#[derive(Debug, Clone, Serialize)]
struct GetResponse {
    #[serde(rename = "saldo")]
    balance: BalanceResponse,
    #[serde(rename = "ultimas_transacoes")]
    last_transactions: Vec<TransactionsResponde>,
}

#[derive(Debug, Clone, Serialize)]
struct BalanceResponse {
    total: i32,
    #[serde(rename = "data_extrato")]
    date: DateTime<Utc>,
    #[serde(rename = "limite")]
    limit: i32,
}

#[derive(Debug, Clone, Serialize)]
struct TransactionsResponde {
    #[serde(rename = "valor")]
    value: i32,
    #[serde(rename = "tipo")]
    kind: TransactionKind,
    #[serde(rename = "descricao")]
    description: String,
    #[serde(rename = "realizada_em")]
    date: DateTime<Utc>,
}

impl From<Transactions> for TransactionsResponde {
    fn from(value: Transactions) -> Self {
        TransactionsResponde {
            value: value.value,
            kind: match value.kind {
                TransactionsKindDb::Credit => TransactionKind::Credit,
                TransactionsKindDb::Debit => TransactionKind::Debit,
            },
            description: value.description,
            date: value.created_at.to_utc(),
        }
    }
}

pub async fn statements_get(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let customer = match query_as!(
        Customers,
        "select * from Customers where id = $1 limit 1",
        id
    )
    .fetch_one(&state.pool)
    .await
    {
        Ok(customers) => customers,
        Err(Error::RowNotFound) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ResponseError {
                    error: String::from("Cliente não existe"),
                }),
            ));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResponseError {
                    error: e.to_string(),
                }),
            ));
        }
    };

    let transactions = match query_as!(
        Transactions,
        r#"select id, value, description, kind as "kind!: TransactionsKindDb", created_at, customer_id from Transactions where customer_id = $1 order by created_at desc limit 10"#,
        id
    )
    .fetch_all(&state.pool)
    .await {
    Ok(transactions) => transactions,
    Err(e) => {
        return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResponseError {
                    error: e.to_string(),
                }),
            ));
        }
    };

    Ok((
        StatusCode::OK,
        Json(GetResponse {
            balance: BalanceResponse {
                total: customer.balance,
                date: Utc::now(),
                limit: customer.account_limit,
            },
            last_transactions: transactions
                .into_iter()
                .map(TransactionsResponde::from)
                .collect(),
        }),
    ))
}
