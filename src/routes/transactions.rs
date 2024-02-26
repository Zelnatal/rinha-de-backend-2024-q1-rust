use axum::{
    extract::{rejection::JsonRejection, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Error};

use crate::{
    config::AppState,
    models::{self, Customers},
};

use super::common::{ResponseError, TransactionKind};

#[derive(Debug, Clone, Deserialize)]
pub struct PostRequest {
    #[serde(rename = "valor")]
    pub value: i32,
    #[serde(rename = "tipo")]
    pub kind: TransactionKind,
    #[serde(rename = "descricao")]
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostResponse {
    #[serde(rename = "limite")]
    limit: i32,
    #[serde(rename = "saldo")]
    balance: i32,
}

pub async fn transaction_post(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    payload: Result<Json<PostRequest>, JsonRejection>,
) -> impl IntoResponse {
    let request_json = match payload {
        Ok(json) => Json(json),
        Err(e) => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ResponseError {
                    error: e.body_text(),
                }),
            ));
        }
    };
    if request_json.value.is_negative() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ResponseError {
                error: String::from("valor está negativo"),
            }),
        ));
    }
    if request_json.description.is_empty() || request_json.description.len() > 10 {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ResponseError {
                error: String::from("descrição tem que ser entre 1 e 10"),
            }),
        ));
    }

    let mut t = match state.pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResponseError {
                    error: e.to_string(),
                }),
            ));
        }
    };

    let customer = match query_as!(
        Customers,
        "select * from Customers where id = $1 limit 1 for update",
        id
    )
    .fetch_one(&mut *t)
    .await
    {
        Ok(customers) => customers,
        Err(Error::RowNotFound) => {
            let _ = t.rollback().await;
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

    if request_json.kind == TransactionKind::Debit
        && (customer.balance - request_json.value) < -customer.account_limit
    {
        let _ = t.rollback().await;
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ResponseError {
                error: String::from("Saldo inconsistente"),
            }),
        ));
    }
    let time = chrono::Local::now();
    let kind = match request_json.kind {
        TransactionKind::Credit => models::TransactionsKindDb::Credit,
        TransactionKind::Debit => models::TransactionsKindDb::Debit,
    };
    match query!(
        "insert into Transactions (value,description,kind,created_at,customer_id) values($1,$2,$3,$4,$5)",
        request_json.value,
        request_json.description,
        kind as models::TransactionsKindDb,
        time,
        id
    ).execute(&mut *t).await {
        Ok(_) => {},
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResponseError {
                    error: e.to_string(),
                }),
            ));
        }
    }

    let new_value = match request_json.kind {
        TransactionKind::Credit => customer.balance + request_json.value,
        TransactionKind::Debit => customer.balance - request_json.value,
    };

    match sqlx::query!(
        "update Customers set balance = $1 where id = $2",
        new_value,
        customer.id
    )
    .execute(&mut *t)
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResponseError {
                    error: e.to_string(),
                }),
            ));
        }
    }

    match t.commit().await {
        Ok(_) => {}
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResponseError {
                    error: e.to_string(),
                }),
            ));
        }
    }

    Ok((
        StatusCode::OK,
        Json(PostResponse {
            limit: customer.account_limit,
            balance: new_value,
        }),
    ))
}
