use sqlx::prelude::FromRow;


#[derive(Debug, Clone, FromRow)]
pub struct Customers {
    pub id: i32,
    pub account_limit: i32,
    pub balance: i32,
}
