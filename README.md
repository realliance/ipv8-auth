# ipv8-auth

```
# Set up your dotenv file for dev
touch .env
cat DATABASE_URL=localhost >> .env
cat DATABASE_USER=postgres >> .env
cat DATABASE_PASS=postgres >> .env
cat DATABASE_DB=postgres >> .env
cat SERVER_URL=127.0.0.1 >> .env
cat SERVER_PORT=3000 >> .env

# Bring up the dev db
sudo docker-compose up -d

# Start the server
cargo run
```
