CREATE TABLE users (
  id UUID PRIMARY KEY,
  name VARCHAR(100) NOT NULL,
  username VARCHAR(100) NOT NULL UNIQUE,
  password_digest VARCHAR NOT NULL,
  license_game_stage INT NOT NULL
)
