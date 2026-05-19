# plutus deployment

Two compose files cover the common cases. Both are Compose Spec compliant, so
`docker compose` and `podman compose` both work — pick whichever you have.

## Local development

Run postgres in a container, the API and web on the host (so you get hot reload).

```bash
docker compose -f deploy/compose.dev.yml up -d
# or: podman compose -f deploy/compose.dev.yml up -d

# In another terminal, with .env populated:
cargo run -p plutus-server -- migrate
cargo run -p plutus-server -- serve

# And a third for the web app:
cd web && pnpm install && pnpm dev
```

## Full stack

```bash
docker compose -f deploy/compose.yml up -d
# or: podman compose -f deploy/compose.yml up -d
```

Visit the web UI at <http://127.0.0.1:3000> and the API at <http://127.0.0.1:8080>.

## Postgres image

`postgres/Dockerfile` layers Apache AGE 1.7.x onto `pgvector/pgvector:pg18`. The
init script enables both extensions on the bootstrap database. To rebuild after
changes:

```bash
docker compose -f deploy/compose.dev.yml build postgres
```

## Resetting state

```bash
docker compose -f deploy/compose.dev.yml down -v   # wipes pgdata
```
