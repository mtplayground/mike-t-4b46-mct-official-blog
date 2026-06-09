# myClawTeam Blog

Leptos + Axum SSR application for the myClawTeam Blog.

## Development

Run the SSR server and hydrating client bundle with cargo-leptos:

```bash
cargo leptos watch
```

The app listens on `0.0.0.0:8080` by default.

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
