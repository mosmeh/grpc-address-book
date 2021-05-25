CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE people (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255)
);

INSERT INTO people (name, email)
VALUES ('Alice Adams', 'alice@example.com');

INSERT INTO people (name)
VALUES ('Bob Baker');

INSERT INTO people (name, email)
VALUES ('Charlie Clark', 'charlie@example.com');
