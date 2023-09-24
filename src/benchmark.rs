use anyhow::Result;
use chrono::Utc;
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use itertools::Itertools;
use std::time::Duration;
use tokio::{
    sync::mpsc,
    time::{sleep, Instant},
};

use crate::{
    routes::{self, HttpMethods},
    utils::median,
};

#[derive(Debug, Clone)]
pub struct Test {
    pub second: i64,
    pub error_code: Option<String>,
    pub response_code: u16,
    pub response_time: u64,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub second: i64,
    pub error_codes: Vec<String>,
    pub response_codes: Vec<u16>,
    pub requests: i64,
    pub avg_response_time: f64,
}

pub async fn run_benchmark(test: routes::CreateTest) -> Result<Vec<(i64, Vec<TestResult>)>> {
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

    let mut results: Vec<TestResult> = vec![];
    let (tx, mut rx) = mpsc::channel(test.tasks as usize);

    // Spawn a tokio task depending on amount of tasks we defined
    for _ in 0..test.tasks {
        let tx_clone = tx.clone();
        let cloned_test = test.clone();

        tokio::spawn(async move {
            let mut thread_results: Vec<Test> = vec![];

            let total_duration = Duration::new(test.seconds, 0);
            let start_time = Instant::now();

            let connector = HttpsConnector::new();
            let client = Client::builder().build(connector);

            while start_time.elapsed() < total_duration {
                let request_start_time = Instant::now();
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

                let mut result = Test {
                    second: second as i64,
                    error_code: None,
                    response_code: 0,
                    response_time: request_start_time.elapsed().as_secs(),
                };

                match resp {
                    Ok(res) => result.response_code = res.status().as_u16(),
                    Err(err) => {
                        result.error_code = Some(err.to_string());
                        // We only add a result if we get a response.
                        // This don't call don't give us a response
                        // Could mean remote server is down
                        log::error!("failed to send request: {:?}", err)
                    }
                }

                thread_results.push(result);
            }

            let mut grouped_results: Vec<TestResult> = vec![];
            for s in 0..test.seconds {
                let test_this_second: Vec<&Test> = thread_results
                    .iter()
                    .filter(|x| x.second == s as i64)
                    .collect_vec();

                let avg_response_time: Vec<f64> = test_this_second
                    .clone()
                    .into_iter()
                    .map(|x| x.response_time as f64)
                    .collect_vec();

                let error_codes = test_this_second
                    .iter()
                    .filter(|x| x.error_code.is_some())
                    .map(|x| x.error_code.clone().unwrap())
                    .collect_vec();

                let response_codes = test_this_second
                    .iter()
                    .map(|x| x.response_code)
                    .collect_vec();

                grouped_results.push(TestResult {
                    second: s as i64,
                    error_codes,
                    requests: test_this_second.len() as i64,
                    avg_response_time: median(avg_response_time),
                    response_codes,
                });
            }

            for r in grouped_results {
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

    let mut grouped_results: Vec<(i64, Vec<TestResult>)> = vec![];

    for s in results.clone().into_iter().map(|x| x.second).unique() {
        let results_per_second: Vec<TestResult> = results
            .clone()
            .into_iter()
            .filter(|x| x.second == s)
            .collect();

        grouped_results.push((s, results_per_second));
    }

    log::info!("finished benchmark");

    Ok(grouped_results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::{CreateTest, HttpMethods};
    use httptest::{matchers::*, responders::*, Expectation, ServerPool};

    // Create a server pool that will create at most 2 servers.
    static SERVER_POOL: ServerPool = ServerPool::new(2);

    #[tokio::test]
    #[should_panic]
    async fn test_run_benchmark_empty_test() {
        let server = SERVER_POOL.get_server();
        server.expect(
            Expectation::matching(request::method_path("GET", "/foo"))
                .times(1..)
                .respond_with(status_code(200)),
        );

        let url = server.url("/foo");

        let test = CreateTest {
            url: url.to_string(),
            method: HttpMethods::GET,
            content_type: None,
            body: None,
            tasks: 0,
            seconds: 0,
            start_at: None,
        };

        let results = run_benchmark(test).await;
        assert!(results.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_run_benchmark_single_task() {
        let server = SERVER_POOL.get_server();
        server.expect(
            Expectation::matching(request::method_path("GET", "/foo"))
                .times(1..)
                .respond_with(status_code(200)),
        );
        let url = server.url("/foo");

        let test = CreateTest {
            url: url.to_string(),
            method: HttpMethods::GET,
            content_type: None,
            body: None,
            tasks: 1,
            seconds: 1,
            start_at: None,
        };

        let results = run_benchmark(test).await.unwrap();
        assert_eq!(results.len(), 1);
        let (second, result) = &results[0];
        assert_eq!(*second, 0);
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_run_benchmark_single_task_many_seconds() {
        let server = SERVER_POOL.get_server();
        server.expect(
            Expectation::matching(request::method_path("GET", "/foo"))
                .times(1..)
                .respond_with(status_code(200)),
        );
        let url = server.url("/foo");

        let test = CreateTest {
            url: url.to_string(),
            method: HttpMethods::GET,
            content_type: None,
            body: None,
            tasks: 1,
            seconds: 10,
            start_at: None,
        };

        let results = run_benchmark(test).await.unwrap();
        assert_eq!(results.len(), 10);
        match &results.last() {
            Some((second, result)) => {
                assert_eq!(*second, 9);
                assert_eq!(result.len(), 1);
            }
            _ => panic!("Should have a result"),
        }
    }
}
