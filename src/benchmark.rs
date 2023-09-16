use crate::routes;

/**
* TODO:
- Run loop for x seconds.
- Report back to main thread the result of test
*/

pub fn run_benchmark(test: routes::CreateTest) {
    for c in 0..test.connections {
        tokio::spawn(async move {
            match test.method.to_uppercase().as_str() {
                "GET" => {
                    let resp = match reqwest::get(test.url).await {
                        Ok(r) => {}
                        Err(err) => {}
                    };
                }
                "POST" => {
                    let client = reqwest::Client::new();
                    let req = client.post(test.url);

                    if test.body.is_some() {
                        req = req.body(test.body); // TODO: only append body if we get body. Also
                                                   // set header depending on content_type
                    }
                }
            };
        });
    }
}
