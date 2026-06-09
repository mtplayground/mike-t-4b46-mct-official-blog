use std::{collections::HashMap, error::Error, fmt};

use axum::{
    Extension, Html, Json,
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::repositories::posts;

const RECENT_POST_LIMIT: i64 = 6;
const POSTS_PER_PAGE: i64 = 9;
const EXCERPT_CHAR_LIMIT: usize = 160;

#[derive(Debug, Serialize)]
pub struct PublicPostCard {
    pub title: String,
    pub slug: String,
    pub category: String,
    pub excerpt: String,
    pub published_at: Option<i64>,
}

#[derive(Debug, Serialize)]
struct ErrorMessage {
    error: &'static str,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    page: Option<i64>,
}

#[derive(Debug)]
pub enum PublicPostsError {
    Database(sqlx::Error),
}

impl fmt::Display for PublicPostsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(_) => write!(f, "failed to load public posts"),
        }
    }
}

impl Error for PublicPostsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
        }
    }
}

impl IntoResponse for PublicPostsError {
    fn into_response(self) -> Response {
        eprintln!("public posts error: {self:?}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorMessage {
                error: "Could not load recent posts. Please try again.",
            }),
        )
            .into_response()
    }
}

pub async fn recent_posts(
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<PublicPostCard>>, PublicPostsError> {
    let categories = posts::list_categories(&pool)
        .await
        .map_err(PublicPostsError::Database)?;
    let category_names = categories
        .into_iter()
        .map(|category| (category.id, category.name))
        .collect::<HashMap<_, _>>();

    let recent_posts = posts::list_published_posts(&pool, RECENT_POST_LIMIT, 0)
        .await
        .map_err(PublicPostsError::Database)?;

    let cards = recent_posts
        .into_iter()
        .map(|post| PublicPostCard {
            title: post.title,
            slug: post.slug,
            category: category_names
                .get(&post.category_id)
                .cloned()
                .unwrap_or_else(|| "Post".to_owned()),
            excerpt: excerpt_from_body(&post.body),
            published_at: post.published_at.map(|published_at| published_at.unix_timestamp()),
        })
        .collect();

    Ok(Json(cards))
}

pub async fn posts_index(
    Extension(pool): Extension<PgPool>,
    Query(query): Query<PaginationQuery>,
) -> Result<Html<String>, PublicPostsError> {
    let total_posts = posts::count_published_posts(&pool)
        .await
        .map_err(PublicPostsError::Database)?;
    let total_pages = total_pages(total_posts);
    let current_page = normalize_page(query.page, total_pages);
    let offset = (current_page - 1) * POSTS_PER_PAGE;

    let categories = posts::list_categories(&pool)
        .await
        .map_err(PublicPostsError::Database)?;
    let category_names = categories
        .into_iter()
        .map(|category| (category.id, category.name))
        .collect::<HashMap<_, _>>();

    let published_posts = posts::list_published_posts(&pool, POSTS_PER_PAGE, offset)
        .await
        .map_err(PublicPostsError::Database)?;

    Ok(Html(render_posts_page(
        &published_posts,
        &category_names,
        current_page,
        total_pages,
        total_posts,
    )))
}

fn excerpt_from_body(body: &str) -> String {
    let normalized = body.split_whitespace().collect::<Vec<_>>().join(" ");

    if normalized.chars().count() <= EXCERPT_CHAR_LIMIT {
        return normalized;
    }

    let mut excerpt = normalized
        .chars()
        .take(EXCERPT_CHAR_LIMIT)
        .collect::<String>();
    excerpt.push_str("...");
    excerpt
}

fn normalize_page(page: Option<i64>, total_pages: i64) -> i64 {
    page.unwrap_or(1).clamp(1, total_pages)
}

fn total_pages(total_posts: i64) -> i64 {
    if total_posts <= 0 {
        1
    } else {
        (total_posts + POSTS_PER_PAGE - 1) / POSTS_PER_PAGE
    }
}

fn render_posts_page(
    posts: &[posts::Post],
    category_names: &HashMap<i64, String>,
    current_page: i64,
    total_pages: i64,
    total_posts: i64,
) -> String {
    let cards = if posts.is_empty() {
        r#"<div class="rounded-lg border border-white/10 bg-surface-900 p-6 text-muted">No published posts yet.</div>"#
            .to_owned()
    } else {
        posts
            .iter()
            .map(|post| render_post_card(post, category_names))
            .collect::<Vec<_>>()
            .join("")
    };
    let pagination = render_pagination(current_page, total_pages);

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>Posts | myClawTeam Blog</title>
<link rel="stylesheet" href="/pkg/mike-t-4b46-mct-official-blog.css" />
</head>
<body>
<div class="min-h-screen bg-background text-foreground antialiased">
<div class="flex min-h-screen flex-col">
<header class="sticky top-0 z-20 border-b border-white/10 bg-background/90 shadow-red-glow backdrop-blur-xl">
<div class="mx-auto flex w-full max-w-6xl flex-wrap items-center justify-between gap-4 px-6 py-4 sm:px-10">
<a href="/" class="group inline-flex items-center gap-3 text-base font-black text-foreground">
<span class="grid h-9 w-9 place-items-center rounded-lg border border-accent-400/40 bg-accent-500 text-sm text-white shadow-red-glow">M</span>
<span class="leading-tight">myClawTeam<span class="block text-xs font-bold uppercase tracking-wide text-accent-400">Blog</span></span>
</a>
<nav aria-label="Primary navigation" class="flex items-center gap-2 text-sm font-bold text-muted">
<a href="/" class="rounded-lg px-3 py-2 transition hover:bg-white/5 hover:text-foreground">Home</a>
<a href="/posts" class="rounded-lg px-3 py-2 text-accent-400 transition hover:bg-white/5 hover:text-foreground">Posts</a>
<a href="/admin" class="rounded-lg border border-accent-400/40 px-3 py-2 text-accent-400 transition hover:bg-accent-500 hover:text-white">Admin</a>
</nav>
</div>
</header>
<main id="content" class="flex-1">
<div class="mx-auto flex w-full max-w-6xl flex-col gap-10 px-6 py-16 sm:px-10 lg:py-20">
<section class="flex flex-col gap-5">
<p class="text-kicker font-bold uppercase tracking-wide text-accent-400">Published posts</p>
<h1 class="max-w-4xl text-display font-black leading-none text-foreground">myClawTeam <span class="text-accent-400">Blog</span></h1>
<p class="max-w-2xl text-lead leading-8 text-muted">A paginated archive of published notes, updates, and delivery writeups.</p>
<p class="text-sm font-bold text-muted">{total_posts} published posts &middot; Page {current_page} of {total_pages}</p>
</section>
<section aria-label="Published post list" class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">{cards}</section>
{pagination}
</div>
</main>
<footer class="border-t border-white/10 bg-background/70">
<div class="mx-auto grid w-full max-w-6xl gap-6 px-6 py-10 text-sm text-muted sm:grid-cols-[1fr_auto] sm:items-end sm:px-10">
<div><p class="text-base font-black text-foreground">myClawTeam Blog</p><p class="mt-2 max-w-xl leading-6">By talking, serious delivery.</p></div>
<nav aria-label="Footer navigation" class="flex flex-wrap gap-3 font-bold">
<a href="/" class="transition hover:text-foreground">Home</a>
<a href="/posts" class="transition hover:text-foreground">Posts</a>
<a href="/admin" class="transition hover:text-accent-400">Admin</a>
</nav>
</div>
</footer>
</div>
</div>
</body>
</html>"#
    )
}

fn render_post_card(post: &posts::Post, category_names: &HashMap<i64, String>) -> String {
    let category = category_names
        .get(&post.category_id)
        .map(String::as_str)
        .unwrap_or("Post");
    let published_at = post
        .published_at
        .map(|value| first_date_chars(&value.to_string()))
        .unwrap_or_else(|| "Published".to_owned());
    let excerpt = excerpt_from_body(&post.body);

    format!(
        r#"<article class="group flex min-h-72 flex-col justify-between rounded-lg border border-white/10 bg-surface-900 p-5 transition hover:-translate-y-1 hover:border-accent-400/60 hover:shadow-red-glow">
<div>
<div class="flex items-center justify-between gap-3">
<p class="text-kicker font-black uppercase tracking-wide text-accent-400">{category}</p>
<span class="h-2 w-2 rounded-full bg-accent-500"></span>
</div>
<h2 class="mt-5 text-2xl font-black leading-tight text-foreground">{title}</h2>
<p class="mt-4 leading-7 text-muted">{excerpt}</p>
</div>
<div class="mt-8 flex items-center justify-between gap-4 border-t border-white/10 pt-4">
<p class="text-xs font-bold uppercase tracking-wide text-muted">{published_at}</p>
<span class="rounded-lg border border-white/10 px-3 py-2 text-sm font-black text-muted">Published</span>
</div>
</article>"#,
        category = escape_html(category),
        title = escape_html(&post.title),
        excerpt = escape_html(&excerpt),
        published_at = escape_html(&published_at),
    )
}

fn render_pagination(current_page: i64, total_pages: i64) -> String {
    if total_pages <= 1 {
        return String::new();
    }

    let previous = if current_page > 1 {
        pagination_link(current_page - 1, "Previous", false)
    } else {
        disabled_pagination_label("Previous")
    };
    let next = if current_page < total_pages {
        pagination_link(current_page + 1, "Next", false)
    } else {
        disabled_pagination_label("Next")
    };
    let pages = (1..=total_pages)
        .map(|page| pagination_link(page, &page.to_string(), page == current_page))
        .collect::<Vec<_>>()
        .join("");

    format!(
        r#"<nav aria-label="Pagination" class="flex flex-wrap items-center justify-between gap-3 border-t border-white/10 pt-6">
<div class="flex gap-2">{previous}{next}</div>
<div class="flex flex-wrap gap-2">{pages}</div>
</nav>"#
    )
}

fn pagination_link(page: i64, label: &str, current: bool) -> String {
    let class = if current {
        "rounded-lg bg-accent-500 px-3 py-2 text-sm font-black text-white"
    } else {
        "rounded-lg border border-white/10 px-3 py-2 text-sm font-black text-foreground transition hover:border-accent-400 hover:text-accent-400"
    };
    let aria_current = if current { r#" aria-current="page""# } else { "" };

    format!(
        r#"<a href="/posts?page={page}" class="{class}"{aria_current}>{label}</a>"#,
        label = escape_html(label),
    )
}

fn disabled_pagination_label(label: &str) -> String {
    format!(
        r#"<span class="rounded-lg border border-white/5 px-3 py-2 text-sm font-black text-muted opacity-50">{}</span>"#,
        escape_html(label)
    )
}

fn first_date_chars(value: &str) -> String {
    value.chars().take(10).collect()
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
