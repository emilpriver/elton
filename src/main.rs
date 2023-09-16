use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

mod database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (s, r) = async_channel::unbounded();

    let connection = match database::setup().await {
        Ok(c) => c,
        Err(err) => panic!("failed to setup database: {:?}", err),
    };

    HttpServer::new(move || {
        App::new()
            .route("/", web::post().to(create_test))
            .route("/:id", web::post().to(get_test))
            .app_data(web::Data::new(connection.clone()))
            .app_data(web::Data::new(tx))
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
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

    for c in 0..payload.connections {
        tokio::spawn(async move || {})
    }

    HttpResponse::Created().json(test)
}

async fn get_test() -> impl Responder {
    let p = Test {
        id: String::from(""),
        method: String::from(""),
        url: String::from(""),
    };

    HttpResponse::Created().json(p)
}
