# myClawTeam Blog

Leptos + Axum SSR application for the myClawTeam Blog.

## Development

Run the SSR server and hydrating client bundle with cargo-leptos:

```bash
cp .env.example .env
# Fill .env with the provisioned values before sourcing it.
set -a
. ./.env
set +a
cargo leptos watch
```

The app listens on `0.0.0.0:8080` by default. On boot it connects to
PostgreSQL, runs migrations from `migrations/`, and verifies connectivity with
`SELECT 1`.

All runtime configuration is read through a typed server-side config loader.
Required variables are documented in `.env.example`.

Tailwind is wired through cargo-leptos with `style/tailwind.css` as the input file.
For standalone CSS checks, use:

```bash
npm install
npm run css:build
```

## Build

```bash
cargo leptos build
```
