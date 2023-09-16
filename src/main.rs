use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, SqlitePool};

mod database;

#[tokio::main]
async fn main() {
    let connection = match database::setup().await {
        Ok(c) => c,
        Err(err) => panic!("failed to setup database: {:?}", err),
    };
    // build our application with a route
    let app = Router::new()
        .layer(Extension(connection))
        .route("/", post(create_test))
        .route("/", post(get_test));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize)]
struct CreateTest {
    pub method: String,
    pub connections: u64,
    pub content_type: String,
    pub url: String,
    pub body: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
struct Test {
    pub id: String,
    pub method: String,
    pub url: String,
}

async fn create_test(
    pool: Extension<SqlitePool>,
    Json(payload): Json<CreateTest>,
) -> (StatusCode, Json<Test>) {
    let mut rng = rand::thread_rng();
    let random_number: u32 = rng.gen_range(100_000..1_000_000);
    let id = random_number.to_string();

    sqlx::query(
        "INSERT INTO tests(id, url, method, content_type, body) VALUES($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(payload.url)
    .bind(payload.method)
    .bind(payload.content_type)
    .bind(payload.body)
    .execute(&pool)
    .await;

    let t = Test {
        id,
        method: payload.method,
        url: payload.url,
    };

    (StatusCode::CREATED, Json(t))
}

async fn get_test() -> (StatusCode, Json<Test>) {
    let p = Test {
        id: String::from(""),
        method: String::from(""),
        url: String::from(""),
    };

    (StatusCode::CREATED, Json(p))
}
