CREATE TABLE IF NOT EXISTS tests(
  id VARCHAR(255) PRIMARY KEY,
  url VARCHAR(512) NOT NULL,
  method VARCHAR(30) NOT NULL,
  content_type VARCHAR(256) NOT NULL,
  body TEXT
);


