# myClawTeam Blog

## What It Is

myClawTeam Blog is a Rust Leptos + Axum SSR blog for publishing myClawTeam posts, delivery notes, announcements, and product progress updates. The public site uses the brand headline "myClawTeam Blog" and slogan "By talking, serious delivery."

## What It Does

- Public homepage with dark editorial styling, highlighted red accents, featured/recent post cards, and newsletter signup.
- Server-rendered post archive with pagination.
- Category-filtered listings for Thoughts, Product Progress, and Announcements.
- Server-rendered post detail pages with title, date, category, rich text, and embedded image/video media.
- Newsletter signup collection and authenticated admin subscriber list.
- Authenticated admin publishing workspace for creating, editing, publishing, unpublishing, and deleting posts.
- Admin media library and post-editor upload flow for images/videos.
- Dynamic page metadata, Open Graph tags, `sitemap.xml`, `robots.txt`, and RSS feed.
- Playwright E2E coverage for admin login, media upload, post publish, and public viewing flow.
- Self-host release scripts for building and running the SSR binary with static assets.

## Architecture

- Rust 2024 application using Leptos for SSR/hydration and Axum for HTTP routes.
- PostgreSQL is the only persistent database. Migrations live in `migrations/` and are run by the app on startup through `sqlx`.
- Object storage uses the AWS S3-compatible SDK with vendor-neutral `OBJECT_STORAGE_*` env vars. All object keys are scoped under `OBJECT_STORAGE_PREFIX`; browser reads use presigned URLs because the bucket is private.
- Admin auth is a server-side username/password flow with an HMAC-signed `mct_admin_session` cookie scoped to `/admin`.
- Public list/detail/feed/sitemap pages are server-rendered. Admin screens use small static JavaScript files for progressive enhancement and JSON API calls.
- Release deployment expects both `target/release/mike-t-4b46-mct-official-blog` and `target/site/`.

## Runtime Conventions

- The app listens on `0.0.0.0:8080` by default via `LEPTOS_SITE_ADDR`.
- Required env vars are documented in `.env.example`: `DATABASE_URL`, admin credentials/session secret, and vendor-neutral Object Storage settings.
- Do not use SQLite, JSON files, in-memory maps, local disk, or ephemeral volumes for persistent state or uploaded media.
- Media upload object keys are stored as relative keys in PostgreSQL; display URLs are generated at read time with presigned GET URLs.
- Build self-hosted artifacts with `./scripts/build-release.sh`; run them with `./scripts/run-release.sh`.
