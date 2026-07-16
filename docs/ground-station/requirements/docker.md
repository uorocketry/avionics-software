# Docker

[Docker](https://docs.docker.com/) is required for services and database. Install [Docker Desktop](https://www.docker.com/products/docker-desktop).

## Start Services

```sh
docker compose up -d
```

## Apply Database Schema

Services connect to LibSQL at `http://db:8080`. Apply schema from host:

```sh
cd db
bun run index.ts
```

Schema (`db/seed.sql`) creates tables: `RadioFrame`, `Command`, SBG sensor tables. Inspect data with LibSQL/Turso-compatible clients or service logs.