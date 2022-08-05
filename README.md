# ipv8-auth

```
# Bring up the dev db
sudo docker-compose up -d

# Install diesel cli
cargo install diesel-cli

# Run DB Migration (for dev)
DATABASE_URL=postgres://postgres:postgres@localhost/postgres diesel migration run

# Run DB Migration (for tests)
DATABASE_URL=postgres://postgres:postgres@localhost:5433/postgres diesel migration run

# Start the server
cargo run

# Run tests
cargo test
```
