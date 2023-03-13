-- Your SQL goes here
CREATE TABLE messages 
(
    timestamp BIGINT PRIMARY KEY,
    username  TEXT        NOT NULL,
    message   TEXT        NOT NULL
)