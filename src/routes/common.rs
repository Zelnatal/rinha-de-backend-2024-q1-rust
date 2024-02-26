use std::fmt::Debug;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ResponseError<T>
where
{
    pub error: T,
}

#[derive(Debug, Clone, PartialEq,Deserialize, Serialize)]
pub enum TransactionKind {
    #[serde(rename = "c")]
    Credit,
    #[serde(rename = "d")]
    Debit,
}
