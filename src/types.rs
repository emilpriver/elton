use chrono::NaiveDateTime;
use core::fmt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, clap::ValueEnum, sqlx::Type)]
#[sqlx(type_name = "method", rename_all = "lowercase")]
pub enum HttpMethods {
    POST,
    GET,
}

impl fmt::Display for HttpMethods {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpMethods::GET => write!(f, "GET"),
            HttpMethods::POST => write!(f, "POST"),
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, clap::ValueEnum)]
pub enum ContentType {
    JSON,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CreateTest {
    pub method: HttpMethods,
    pub tasks: u64,
    pub seconds: u64,
    pub start_at: Option<NaiveDateTime>,
    pub url: String,
    pub content_type: Option<ContentType>,
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
