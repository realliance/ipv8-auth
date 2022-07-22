-- Your SQL goes here
CREATE TABLE games (
  user_id UUID PRIMARY KEY,
  token UUID NOT NULL,
  instruction INT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users (id)
)
