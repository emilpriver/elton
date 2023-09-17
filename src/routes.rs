use actix_web::{get, post, web, HttpResponse, Responder};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::benchmark;
use crate::benchmark::Result;
use crate::database::{self, TestResultsRow};

#[derive(Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "method", rename_all = "lowercase")]
pub enum HttpMethods {
    POST,
    GET,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CreateTest {
    pub method: HttpMethods,
    pub connections: u64,
    pub threads: u64,
    pub seconds: u64,
    pub start_at: String,
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

#[post("/")]
pub async fn create_test(
    pool: web::Data<Pool<Sqlite>>,
    payload: web::Json<CreateTest>,
) -> impl Responder {
    let id = Uuid::new_v4().to_string();

    let test: database::TestRow = match sqlx::query_as(
        "INSERT INTO tests(id, url, method, content_type, body) VALUES($1, $2, $3, $4, $5) RETURNING id, url, method, content_type, status, body, finished_at, created_at",
    )
    .bind(&id)
    .bind(&payload.url)
    .bind(&payload.method)
    .bind(&payload.content_type)
    .bind(&payload.body)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(t) => t,
        Err(err) => {
            log::error!("failed to insert test into database: {:?}", err);
            return HttpResponse::InternalServerError().body("failed to insert test into database, check logs for more information");
        }
    };

    tokio::task::spawn(async move {
        let benchmark_result = benchmark::run_benchmark(payload.0).await;

        let mut grouped_results: Vec<(i64, Vec<Result>)> = vec![];

        for s in benchmark_result
            .clone()
            .into_iter()
            .map(|x| x.second)
            .unique()
        {
            let results_per_second: Vec<benchmark::Result> = benchmark_result
                .clone()
                .into_iter()
                .filter(|x| x.second == s)
                .collect();

            grouped_results.push((s, results_per_second));
        }

        for (sec, r) in grouped_results {
            let total_requests: i64 = r.clone().into_iter().map(|x| x.requests).sum();
            let error_codes: Vec<String> = r
                .into_iter()
                .flat_map(|x| x.error_codes)
                .map(|x| x.to_string())
                .collect::<Vec<String>>();

            let test_id = Uuid::new_v4();

            match sqlx::query(
                "INSERT INTO test_results(id, test_id, second, requests, error_codes) VALUES($1, $2, $3, $4, $5)",
            )
            .bind(test_id.to_string())
            .bind(&id)
            .bind(sec)
            .bind(total_requests)
            .bind(error_codes.join(","))
            .execute(pool.get_ref())
            .await {
                Err(err) => {
                        log::error!("error inserting test_results: {:?}", err)
                    }
                _ => {},
            }
        }

        match sqlx::query(
            "UPDATE tests SET status = 'FINISHED', finished_at = CURRENT_TIMESTAMP WHERE id = $1",
        )
        .bind(&id)
        .execute(pool.get_ref())
        .await
        {
            Err(err) => {
                log::error!("error inserting test_results: {:?}", err)
            }
            _ => {}
        }
    });

    HttpResponse::Created().json(test)
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct TestWithResults {
    id: String,
    url: String,
    method: String,
    content_type: String,
    status: String,
    body: Option<String>,
    created_at: String,
    finished_at: Option<String>,
    resuluts: Vec<TestResultsRow>,
}

#[get("/{test_id}")]
pub async fn get_test(pool: web::Data<Pool<Sqlite>>, test_id: web::Path<String>) -> impl Responder {
    let test: database::TestRow  = match sqlx::query_as("SELECT id, url, method, content_type, status, body, finished_at, created_at FROM tests WHERE id = $1")
        .bind(&test_id.as_str())
        .fetch_one(pool.get_ref())
        .await
        {
        Ok(r) => r,
        Err(err) => {
            println!("error fetching test: {:?}", err);

            return HttpResponse::NotFound().body("not found")
        }
    };

    let test_results: Vec<database::TestResultsRow> = match sqlx::query_as(
        "SELECT id, test_id, second, requests, error_codes FROM test_results WHERE test_id = $1",
    )
    .bind(&test_id.as_str())
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(r) => r,
        Err(err) => {
            println!("error fetching test: {:?}", err);

            return HttpResponse::NotFound().body("not found");
        }
    };

    let resp = TestWithResults {
        id: test.id,
        url: test.url,
        method: test.method,
        content_type: test.content_type,
        status: test.status,
        body: test.body,
        created_at: test.created_at,
        finished_at: test.finished_at,
        resuluts: test_results,
    };

    HttpResponse::Ok().json(resp)
}
