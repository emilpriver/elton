CREATE TABLE IF NOT EXISTS tests(
  id VARCHAR(255) PRIMARY KEY,
  url VARCHAR(512) NOT NULL,
  method VARCHAR(30) NOT NULL,
  content_type VARCHAR(100) NOT NULL,
  status VARCHAR(100) DEFAULT "PROCESSING" NOT NULL,
  body TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  finished_at TIMESTAMP
);

CREATE TABLE IF NOT EXISTS test_results (
  id VARCHAR(255) PRIMARY KEY,
  test_id VARCHAR(255),
  second NUMBER,
  avg_response_time NUMBER,
  requests NUMBER,
  error_codes TEXT
  response_codes TEXT
);


