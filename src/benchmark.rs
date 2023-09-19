use chrono::Utc;
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use std::time::Duration;
use tokio::{
    sync::mpsc,
    time::{sleep, Instant},
};

use crate::routes::{self, HttpMethods};

#[derive(Debug, Clone)]
pub struct Result {
    pub thread_id: u64,
    pub second: i64,
    pub error_codes: Vec<String>,
    pub requests: i64,
}

pub async fn run_benchmark(test: routes::CreateTest) -> Vec<Result> {
    log::info!(
        "Starting benchmark using {} tasks for {} seconds",
        test.tasks,
        test.seconds
    );

    // if we have a start_at timestamp do we wait until start_at is elapsed and then run the
    // benchmark
    if let Some(start_at) = test.start_at {
        let time_now = Utc::now();
        let time_until_start_at = start_at.and_utc() - time_now;

        sleep(Duration::from_micros(
            time_until_start_at.num_microseconds().unwrap_or(0) as u64,
        ))
        .await;
    }

    let mut results: Vec<Result> = vec![];
    let (tx, mut rx) = mpsc::channel(test.tasks as usize);

    for thread in 0..test.tasks {
        let tx_clone = tx.clone();
        let cloned_test = test.clone();

        tokio::spawn(async move {
            let mut total_result: Vec<Result> = vec![];

            for s in 0..test.seconds {
                total_result.push(Result {
                    thread_id: thread,
                    second: s as i64,
                    error_codes: vec![],
                    requests: 0,
                })
            }

            let total_duration = Duration::new(test.seconds, 0);
            let start_time = Instant::now();

            let connector = HttpsConnector::new();
            let client = Client::builder().build(connector);

            while start_time.elapsed() < total_duration {
                let second = start_time.elapsed().as_secs() as usize;

                let resp = match &cloned_test.method {
                    HttpMethods::GET => client.get(cloned_test.url.clone().parse().unwrap()).await,
                    HttpMethods::POST => {
                        let mut req = Request::post(cloned_test.url.clone());
                        if let Some(c) = &cloned_test.content_type {
                            req = req.header("Content-Type", c);
                        }

                        let mut body = Body::empty();
                        if let Some(b) = cloned_test.body.clone() {
                            body = Body::from(b);
                        }

                        let new_request = match req.body(body) {
                            Ok(nr) => nr,
                            Err(err) => {
                                // If we are not able to create the body do we log a request and
                                // break the loop
                                log::error!("failed to add body to request: {:?}", err);
                                break;
                            }
                        };

                        client.request(new_request).await
                    }
                };

                if total_result.get(second).is_some() {
                    total_result[second].requests += 1;

                    match resp {
                        Ok(res) => {
                            if !res.status().is_success() {
                                total_result[second]
                                    .error_codes
                                    .push(res.status().to_string());
                            }
                        }
                        Err(err) => {
                            total_result[second].error_codes.push(err.to_string());
                            // We only add a result if we get a response.
                            // This don't call don't give us a response
                            // Could mean remote server is down
                            log::error!("failed to send request: {:?}", err)
                        }
                    }
                }
            }

            for r in total_result {
                match tx_clone.send(r).await {
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

        if results.len() >= (test.tasks * test.seconds) as usize {
            break;
        }
    }

    log::info!("finished benchmark");

    results
}
