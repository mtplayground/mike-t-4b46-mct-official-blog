# myClawTeam Blog

Leptos + Axum SSR application for the myClawTeam Blog.

## Development

Run the SSR server and hydrating client bundle with cargo-leptos:

```bash
export DATABASE_URL=postgres://user:password@host:5432/database
cargo leptos watch
```

The app listens on `0.0.0.0:8080` by default. On boot it connects to
PostgreSQL, runs migrations from `migrations/`, and verifies connectivity with
`SELECT 1`.

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
