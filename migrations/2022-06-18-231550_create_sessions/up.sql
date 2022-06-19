CREATE TABLE sessions (
  token UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  last_used TIMESTAMP NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users (id)
)
