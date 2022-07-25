# ipv8-auth

```
# Set up your dotenv file for dev
touch .env
cat DATABASE_URI=localhost >> .env
cat DATABASE_USER=postgres >> .env
cat DATABASE_PASS=postgres >> .env
cat DATABASE_DB=postgres >> .env
cat SERVER_URL=127.0.0.1 >> .env
cat SERVER_PORT=3000 >> .env

# Bring up the dev db
sudo docker-compose up -d

# Install diesel cli
cargo install diesel-cli

# Run DB Migration (for dev)
DATABASE_URL=postgres://postgres:postgres@localhost/postgres diesel migration run

# Start the server
cargo run
```
