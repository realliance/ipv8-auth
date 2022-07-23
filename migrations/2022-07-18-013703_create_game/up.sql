-- Your SQL goes here
CREATE TABLE games (
  user_id UUID PRIMARY KEY,
  token UUID NOT NULL,
  instruction INT NOT NULL,
  contacted_fizz BOOLEAN NOT NULL,
  contacted_buzz BOOLEAN NOT NULL,
  contacted_instructions BOOLEAN NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users (id)
)
