use chrono::{DateTime, Local};
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Transactions {
    pub id: i32,
    pub value: i32,
    pub description: String,
    pub kind: TransactionsKindDb,
    pub created_at: DateTime<Local>,
    pub customer_id: i32,
}

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "transactions_kind", rename_all = "lowercase")]
pub enum TransactionsKindDb {
    Credit,
    Debit,
}
