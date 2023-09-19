# Elton

Elton is an HTTP Benchmark utility with options to be used within an HTTP interface.

## Installation

Elton is currently available via Docker or by cloning this git repo.

### Docker

The easiest way to run Elton with an HTTP interface is to use Docker.

```bash
docker run -p 8080:3000 emilpriver/elton:v0.0.2

```

This command runs Elton and exposes port 8080. Elton should now be accessible via `http://localhost:8080`.

### Building from source

1. Clone this repository using Git.
2. Run `cargo build -r`.
3. Elton can now be run using `./target/release/elton`.

## Running tests using HTTP

Elton currently exposes 2 endpoints to start and get the status of a test.

### Initialize a test

```powershell
curl --location 'localhost:3000/' \\
--header 'Content-Type: application/json' \\
--data '{
    "method": "GET", // GET or POST
    "tasks": 1000, // Number of tasks to spin up
    "seconds": 10, // Test duration
    "start_at": "2023-09-17T10:16:34.675Z", // Start at timestamp
    "url": "<https://httpbin.org/ip>", // URL to test
    "content_type": "application/json", // Content-Type to send POST message using
    "body": "test" // POST BODY
}'

```

The response includes the following JSON:

```json
{
    "id": "ee7e44cc-b35e-4752-a94d-32adcb17dfc2", // Test ID
    "url": "<https://httpbin.org/ip>",
    "method": "get",
    "content_type": "application/json",
    "status": "PROCESSING",
    "body": "test",
    "created_at": "2023-09-19 07:44:12",
    "finished_at": null
}

```

### Check test status

You can fetch the result from a test by calling `/{test_id}`.

```powershell
curl --location --request GET 'localhost:3000/ee7e44cc-b35e-4752-a94d-32adcb17dfc2'

```

This response returns:

```json
{
    "id": "d9dbf273-0b5d-42bb-8be5-eaef2513a03a",
    "url": "<https://httpbin.org/ip>",
    "method": "get",
    "content_type": "application/json",
    "status": "PROCESSING",
    "body": "test",
    "created_at": "2023-09-19 07:46:47",
    "finished_at": null,
    "results": [
        {
            "id": "173e17f5-8909-4940-9182-a0ed2f434133",
            "test_id": "d9dbf273-0b5d-42bb-8be5-eaef2513a03a",
            "second": 0, // The second this amount of requests was sent at.
            "requests": 1639, // Number of requests done this second
            "error_codes": "Socket Hungup" // If there is an error, the error codes are stored here
        }
    ]
}
```
