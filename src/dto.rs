use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub(crate) struct PersonalResponse {
    pub(crate) result: HashMap<String, PersonalDayProfit>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub(crate) struct PersonalDayProfit {
    pub(crate) net_profit: ProfitData,
    pub(crate) revenue: ProfitData,
}

#[derive(Deserialize, Debug)]
pub(crate) struct ProfitData {
    pub(crate) data: HashMap<String, f32>,
}

#[derive(Debug)]
pub(crate) struct ProfitDay {
    pub(crate) date: chrono::NaiveDate,
    pub(crate) value: f32,
}
