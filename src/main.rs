use actix_web::{web, App, HttpServer};
use routes::{create_test, get_test};
use simple_logger::SimpleLogger;

mod benchmark;
mod database;
mod routes;
mod utils;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    match SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
    {
        Err(err) => {
            println!("failed to setup logging: {:?}", err);
        }
        _ => {}
    }

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
