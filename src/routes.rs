use actix_web::{web, HttpResponse, Responder};
use rand::{self, Rng};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::benchmark;

#[derive(Deserialize)]
pub struct CreateTest {
    pub method: String,
    pub connections: u64,
    pub seconds: u64,
    pub start_at: String, // TODO: change this to chrono timestamp
    pub content_type: String,
    pub url: String,
    pub body: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Test {
    pub id: String,
    pub method: String,
    pub url: String,
}

pub async fn create_test(
    pool: web::Data<SqlitePool>,
    payload: web::Json<CreateTest>,
) -> impl Responder {
    let mut rng = rand::thread_rng();
    let random_number: u32 = rng.gen_range(100_000..1_000_000);
    let id = random_number.to_string();

    let test: Test = sqlx::query_as(
        "INSERT INTO tests(id, url, method, content_type, body) VALUES($1, $2, $3, $4, $5) RETURNING id, method, url",
    )
    .bind(id.clone())
    .bind(payload.url.clone())
    .bind(payload.method.clone())
    .bind(payload.content_type.clone())
    .bind(payload.body.clone())
    .fetch_one(&mut pool)
    .await
    .unwrap();

    benchmark::run_benchmark(payload.0);

    HttpResponse::Created().json(test)
}

pub async fn get_test() -> impl Responder {
    let p = Test {
        id: String::from(""),
        method: String::from(""),
        url: String::from(""),
    };

    HttpResponse::Created().json(p)
}
