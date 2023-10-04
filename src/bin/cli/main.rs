extern crate modules;

use modules::{
    benchmark,
    types::{self, ContentType, HttpMethods},
};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = HttpMethods::GET)]
    method: HttpMethods,
    #[arg(short, long, default_value_t = 10)]
    tasks: u64,
    #[arg(short, long, default_value_t = 10)]
    seconds: u64,
    #[arg(short, long)]
    url: String,
    #[arg(short, long)]
    content_type: Option<ContentType>,
    #[arg(short, long)]
    body: Option<String>,
}

#[tokio::main]
pub async fn main() {
    let args = Args::parse();

    let t = types::CreateTest {
        method: args.method,
        tasks: args.tasks,
        seconds: args.seconds,
        start_at: None,
        url: args.url,
        content_type: args.content_type,
        body: args.body,
    };

    let result = benchmark::run_benchmark(t).await;

    println!("{:?}", result)
}
