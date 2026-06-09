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
Object Storage uses the vendor-neutral `OBJECT_STORAGE_*` variables and always
prepends `OBJECT_STORAGE_PREFIX` before S3 operations. Browser-facing object
reads should use presigned GET URLs from the storage helper because the bucket is
private.

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
