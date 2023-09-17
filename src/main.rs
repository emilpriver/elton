use actix_web::{web, App, HttpServer};
use routes::{create_test, get_test};

mod benchmark;
mod database;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let connection = match database::setup().await {
        Ok(c) => c,
        Err(err) => panic!("failed to setup database: {:?}", err),
    };

    HttpServer::new(move || {
        App::new()
            .service(get_test)
            .service(create_test)
            .app_data(web::Data::new(connection.clone()))
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}
