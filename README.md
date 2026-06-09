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

## Self-Hosted Release

The production server is a single Axum/Leptos SSR binary that serves dynamic
routes, API routes, and static assets from the Leptos site root. A self-hosted
deployment needs both:

- `target/release/mike-t-4b46-mct-official-blog`
- `target/site/`

Build them together with:

```bash
./scripts/build-release.sh
```

The script requires Rust, `cargo-leptos`, Node.js, and npm. It installs npm
dependencies from the lockfile, runs `cargo leptos build --release`, and
verifies that the release binary plus static site bundle were produced.

Run the release locally or on a self-hosted VM with:

```bash
cp .env.example .env.production
# Fill .env.production with provisioned values.
./scripts/run-release.sh
```

`scripts/run-release.sh` loads `.env.production` by default, sets
`LEPTOS_SITE_ADDR=0.0.0.0:8080` and `LEPTOS_SITE_ROOT=target/site` when they are
not already set, then executes the release binary. To use a different env file:

```bash
ENV_FILE=/etc/myclawteam-blog.env ./scripts/run-release.sh
```

### Required Environment Variables

Database:

- `DATABASE_URL`: PostgreSQL connection string. The app runs embedded
  migrations on startup and does not support SQLite or file-backed persistence.

Admin publishing:

- `ADMIN_USERNAME`: username for `/admin/login`.
- `ADMIN_PASSWORD`: password for `/admin/login`.
- `SESSION_SECRET`: at least 32 bytes; signs the admin session cookie.

Object Storage:

- `OBJECT_STORAGE_ACCESS_KEY_ID`
- `OBJECT_STORAGE_SECRET_ACCESS_KEY`
- `OBJECT_STORAGE_BUCKET`
- `OBJECT_STORAGE_PREFIX`: must be non-empty and end with `/`. Every object key
  is scoped under this prefix before S3 operations.
- `OBJECT_STORAGE_ENDPOINT`
- `OBJECT_STORAGE_REGION`: use `auto` for Tigris.
- `OBJECT_STORAGE_FORCE_PATH_STYLE`: use `true` for Tigris.

Leptos runtime:

- `LEPTOS_SITE_ADDR`: listener address, default `0.0.0.0:8080`.
- `LEPTOS_SITE_ROOT`: static asset root, default `target/site`.

For a portable release artifact, copy the binary and `target/site/` together and
start the binary from the repository root or set `LEPTOS_SITE_ROOT` to the copied
site directory.
