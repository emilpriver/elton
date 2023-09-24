mod benchmark;
mod cli;
mod database;
mod utils;

mod routes;
mod web;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    cfg_if::cfg_if! {
        if #[cfg(target_feature = "cli")] {
            cli::run();
        }else {
            web::run_web_app().await;
        }
    }

    Ok(())
}
