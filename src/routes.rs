use actix_web::{web, HttpResponse, Responder};
use rand::{self, Rng};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

use crate::benchmark;

#[derive(Deserialize, Clone)]
pub struct CreateTest {
    pub method: String,
    pub connections: u64,
    pub seconds: u64,
    pub start_at: String, // TODO: change this to chrono timestamp
    pub url: String,
    pub content_type: Option<String>,
    pub body: Option<String>,
}

#[derive(Serialize, Clone, sqlx::FromRow)]
pub struct Test {
    pub id: String,
    pub method: String,
    pub url: String,
}

pub async fn create_test(
    pool: web::Data<Pool<Sqlite>>,
    payload: web::Json<CreateTest>,
) -> impl Responder {
    let mut rng = rand::thread_rng();
    let random_number: u32 = rng.gen_range(100_000..1_000_000);
    let id = random_number.to_string();

    let db_pool = pool.get_ref();

    let test: Test = sqlx::query_as(
        "INSERT INTO tests(id, url, method, content_type, body) VALUES($1, $2, $3, $4, $5) RETURNING id, method, url",
    )
    .bind(id.clone())
    .bind(payload.url.clone())
    .bind(payload.method.clone())
    .bind(payload.content_type.clone())
    .bind(payload.body.clone())
    .fetch_one(db_pool)
    .await
    .unwrap();

    benchmark::run_benchmark(payload.0);

    HttpResponse::Created().json(test)
}

pub async fn get_test(pool: web::Data<Pool<Sqlite>>) -> impl Responder {
    let p = Test {
        id: String::from(""),
        method: String::from(""),
        url: String::from(""),
    };

    HttpResponse::Created().json(p)
}
