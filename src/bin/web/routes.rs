use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use modules::benchmark;
use modules::database::{self, TestResultsRow};
use modules::types::{ContentType, CreateTest};
use modules::utils::median;

#[post("/")]
pub async fn create_test(
    pool: web::Data<Pool<Sqlite>>,
    payload: web::Json<CreateTest>,
) -> impl Responder {
    let id = Uuid::new_v4().to_string();

    let content_type: Option<String> = match &payload.content_type {
        Some(ContentType::JSON) => Some(String::from("application/json")),
        None => None,
    };

    let test: database::TestRow = match sqlx::query_as(
        "INSERT INTO tests(id, url, method, content_type, body) VALUES($1, $2, $3, $4, $5) RETURNING id, url, method, content_type, status, body, finished_at, created_at",
    )
    .bind(&id)
    .bind(&payload.url)
    .bind(&payload.method)
    .bind(content_type)
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
        let benchmark_result = match benchmark::run_benchmark(payload.0).await {
            Ok(b) => b,
            Err(err) => {
                log::error!("failed to run test: {:?}", err);
                return match sqlx::query(
                    "UPDATE tests SET status = 'FAILED', finished_at = CURRENT_TIMESTAMP WHERE id = $1",
                )
                .bind(&id)
                .execute(pool.get_ref())
                .await
                {
                    Err(err) => {
                        log::error!("error inserting test_results: {:?}", err)
                    }
                    _ => {}
                };
            }
        };

        for (sec, r) in benchmark_result {
            let total_requests: i64 = r.clone().into_iter().map(|x| x.requests).sum();
            let avg_response_time: Vec<f64> =
                r.clone().into_iter().map(|x| x.avg_response_time).collect();

            let media_avg_response_time = median(avg_response_time);

            let error_codes: Vec<String> = r
                .clone()
                .into_iter()
                .flat_map(|x| x.error_codes)
                .map(|x| x.to_string())
                .collect::<Vec<String>>();

            let response_codes: Vec<String> = r
                .into_iter()
                .flat_map(|x| x.response_codes)
                .map(|x| x.to_string())
                .collect::<Vec<String>>();

            let test_id = Uuid::new_v4();

            match sqlx::query(
                "
                INSERT INTO test_results(id, test_id, second, requests, error_codes, avg_response_time, response_codes) 
                VALUES($1, $2, $3, $4, $5, $6, $7)
                ",
            )
            .bind(test_id.to_string())
            .bind(&id)
            .bind(sec)
            .bind(total_requests)
            .bind(media_avg_response_time)
            .bind(response_codes.join(","))
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
    results: Vec<TestResultsRow>,
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
        "SELECT id, test_id, second, requests, error_codes, response_codes, avg_response_time FROM test_results WHERE test_id = $1",
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
        results: test_results,
    };

    HttpResponse::Ok().json(resp)
}
