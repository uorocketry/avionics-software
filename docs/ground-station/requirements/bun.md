# Bun

Bun is installed via `devbox`. For Bun applications, install dependencies and run:

```sh
cd <app-directory>
bun install
bun run dev
```

Test with the `web` folder. Server runs at <http://localhost:3000/>.

> Tip: Browser warnings about untrusted certificates in local dev are safe to ignore.

## Troubleshooting

**Common mistakes:**
- Running `npm install` → Use `bun install` instead
- Running as `sudo` → Never run package managers as sudo

**ENOENT errors:** Verify you're in the correct directory.

**Config/module errors:** Run `bun install` then `bun run dev` again.

**Connection errors:** Ensure the dev server is running and check the URL.
