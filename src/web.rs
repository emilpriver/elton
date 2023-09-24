use crate::database;
use crate::routes::{create_test, get_test};
use actix_web::{web, App, HttpServer};
use simple_logger::SimpleLogger;

pub async fn run_web_app() {
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

    let server = match HttpServer::new(move || {
        App::new()
            .service(get_test)
            .service(create_test)
            .app_data(web::Data::new(connection.clone()))
    })
    .bind(("0.0.0.0", 3000))
    {
        Ok(s) => s,
        Err(err) => panic!("failed to create server: {:?}", err),
    };

    server.run().await.unwrap();
}
