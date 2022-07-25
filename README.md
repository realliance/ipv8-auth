# ipv8-auth

```
# Set up your dotenv file for dev
cat DATABASE_URL=postgres://postgres:postgres@localhost/postgres > .env
cat SERVER_URL=127.0.0.1 >> .env
cat SERVER_PORT=3000 >> .env

# Bring up the dev db
sudo docker-compose up -d

# Start the server
cargo run
```
