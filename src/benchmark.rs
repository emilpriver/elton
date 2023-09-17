use reqwest::StatusCode;
use tokio::{
    sync::mpsc,
    time::{Duration, Instant},
};

use crate::routes;

#[derive(Debug, Clone)]
pub struct Result {
    pub connection_id: u64,
    pub second: i64,
    pub error_codes: Vec<StatusCode>,
    pub requests: i64,
}

// TODO: response logging progres
pub async fn run_benchmark(test: routes::CreateTest) -> Vec<Result> {
    let mut results: Vec<Result> = vec![];
    let connections_usize = test.connections.clone() as usize;

    let (tx, mut rx) = mpsc::channel(connections_usize);

    for c in 0..test.connections {
        let thread_test = test.clone();
        let cloned_tx = tx.clone();

        tokio::task::spawn(async move {
            let mut total_result: Vec<Result> = vec![];

            // We need to fill the vec
            for s in 0..test.seconds {
                total_result.push(Result {
                    connection_id: c,
                    second: s as i64,
                    error_codes: vec![],
                    requests: 0,
                })
            }

            let total_duration = Duration::new(test.seconds, 0);
            let start_time = Instant::now();

            while start_time.elapsed() < total_duration {
                let second = start_time.elapsed().as_secs() as usize;

                let resp = match thread_test.method.to_uppercase().as_str() {
                    "GET" => reqwest::get(thread_test.url.clone()).await,
                    "POST" => {
                        let client = reqwest::Client::new();
                        let mut req = client.post(thread_test.url.clone());

                        if let Some(b) = thread_test.body.clone() {
                            req = req.body(b);
                        }

                        if let Some(c) = thread_test.content_type.clone() {
                            req = req.header("Content-Type", c);
                        }

                        req.send().await
                    }
                    _ => {
                        panic!("method {} not supported", thread_test.method)
                    }
                };

                if total_result.get(second).is_some() {
                    total_result[second].requests += 1;

                    match resp {
                        Ok(res) => {
                            if !res.status().is_success() {
                                total_result[second].error_codes.push(res.status());
                            }
                        }
                        Err(err) => {
                            if let Some(err_status) = err.status() {
                                total_result[second].error_codes.push(err_status);
                            }
                        }
                    }
                }
            }

            for r in total_result {
                match cloned_tx.send(r).await {
                    Err(err) => {
                        log::error!("failed to send result to channel: {:?}", err)
                    }
                    _ => {} //Ignore if sending to channel was ok
                }
            }
        });
    }

    while let Some(i) = rx.recv().await {
        results.push(i);

        if results.len() >= (test.connections * test.seconds) as usize {
            break;
        }
    }

    results
}
