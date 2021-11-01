use super::schema::{configs, net_profits, revenues};

#[derive(Insertable, Queryable)]
pub struct NetProfit {
    pub(crate) day: chrono::NaiveDate,
    pub(crate) value: f32,
}

#[derive(Insertable, Queryable)]
pub struct Revenue {
    pub(crate) day: chrono::NaiveDate,
    pub(crate) value: f32,
}

#[derive(Insertable, Queryable)]
pub struct Config {
    pub(crate) key: String,
    pub(crate) value: String,
}
