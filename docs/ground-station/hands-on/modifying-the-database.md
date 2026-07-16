# Modifying the Database

Schema is defined in `deployments/ground-station/db/seed.sql` and applied by its Bun migration script.

**Steps:**

1. Edit `deployments/ground-station/db/seed.sql`.
2. Start LibSQL instance (see [Docker requirements](../requirements/docker.md))
3. Apply schema:

```sh
cd deployments/ground-station/db
bun run index.ts
```

On failure, the script prints the failed SQL and exits non-zero.
