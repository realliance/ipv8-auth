# ipv8-auth
[![Docker](https://github.com/realliance/ipv8-auth/actions/workflows/docker-publish.yml/badge.svg)](https://github.com/realliance/ipv8-auth/actions/workflows/docker-publish.yml)
[![codecov](https://codecov.io/gh/realliance/ipv8-auth/branch/main/graph/badge.svg?token=RMMYVWK1UO)](https://codecov.io/gh/realliance/ipv8-auth)

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
