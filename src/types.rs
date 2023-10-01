use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "method", rename_all = "lowercase")]
pub enum HttpMethods {
    POST,
    GET,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CreateTest {
    pub method: HttpMethods,
    pub tasks: u64,
    pub seconds: u64,
    pub start_at: Option<NaiveDateTime>,
    pub url: String,
    pub content_type: Option<String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Test {
    pub second: i64,
    pub error_code: Option<String>,
    pub response_code: u16,
    pub response_time: u64,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub second: i64,
    pub error_codes: Vec<String>,
    pub response_codes: Vec<u16>,
    pub requests: i64,
    pub avg_response_time: f64,
}
