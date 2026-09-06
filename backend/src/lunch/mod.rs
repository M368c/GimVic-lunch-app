use chrono::NaiveDate;

pub mod lunch_data;
pub mod update_data;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LunchData {
    status: String,
    date: NaiveDate,
}
