use std::{collections::HashMap, error::Error, fmt, time::Duration};

use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    repositories::{media, posts},
    storage::{ObjectStorage, StorageError},
};

const RECENT_POST_LIMIT: i64 = 6;
const POSTS_PER_PAGE: i64 = 9;
const EXCERPT_CHAR_LIMIT: usize = 160;
const PRESIGNED_MEDIA_TTL: Duration = Duration::from_secs(60 * 60);
const SITE_NAME: &str = "myClawTeam Blog";
const OG_IMAGE_PATH: &str = "/og-card.svg";
const CATEGORY_FILTERS: [CategoryFilter; 3] = [
    CategoryFilter {
        slug: "thoughts",
        name: "Thoughts",
        description: "Notes and reflections from the myClawTeam team.",
    },
    CategoryFilter {
        slug: "product-progress",
        name: "Product Progress",
        description: "Updates on what is changing and shipping.",
    },
    CategoryFilter {
        slug: "announcements",
        name: "Announcements",
        description: "Official news and milestones.",
    },
];

#[derive(Clone, Copy, Debug)]
struct CategoryFilter {
    slug: &'static str,
    name: &'static str,
    description: &'static str,
}

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
    RecentPostsDatabase(sqlx::Error),
    Storage(StorageError),
    PostNotFound(String),
    CategoryNotFound(String),
}

impl fmt::Display for PublicPostsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(_) => write!(f, "failed to load public posts"),
            Self::RecentPostsDatabase(_) => write!(f, "failed to load recent posts"),
            Self::Storage(_) => write!(f, "failed to sign embedded media"),
            Self::PostNotFound(slug) => write!(f, "post not found: {slug}"),
            Self::CategoryNotFound(slug) => write!(f, "category filter not found: {slug}"),
        }
    }
}

impl Error for PublicPostsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) | Self::RecentPostsDatabase(error) => Some(error),
            Self::Storage(error) => Some(error),
            Self::PostNotFound(_) => None,
            Self::CategoryNotFound(_) => None,
        }
    }
}

impl IntoResponse for PublicPostsError {
    fn into_response(self) -> Response {
        eprintln!("public posts error: {self:?}");
        match self {
            Self::RecentPostsDatabase(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorMessage {
                    error: "Could not load recent posts. Please try again.",
                }),
            )
                .into_response(),
            Self::Database(_) | Self::Storage(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error_page(
                    "Could not load content",
                    "The page could not be loaded. Please try again.",
                )),
            )
                .into_response(),
            Self::CategoryNotFound(slug) => (
                StatusCode::NOT_FOUND,
                Html(render_not_found_page(
                    "Category not found",
                    &format!("Category filter '{}' was not found.", slug),
                )),
            )
                .into_response(),
            Self::PostNotFound(slug) => (
                StatusCode::NOT_FOUND,
                Html(render_not_found_page(
                    "Post not found",
                    &format!("Post '{}' was not found.", slug),
                )),
            )
                .into_response(),
        }
    }
}

pub async fn recent_posts(
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<PublicPostCard>>, PublicPostsError> {
    let categories = posts::list_categories(&pool)
        .await
        .map_err(PublicPostsError::RecentPostsDatabase)?;
    let category_names = categories
        .into_iter()
        .map(|category| (category.id, category.name))
        .collect::<HashMap<_, _>>();

    let recent_posts = posts::list_published_posts(&pool, RECENT_POST_LIMIT, 0)
        .await
        .map_err(PublicPostsError::RecentPostsDatabase)?;

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

pub async fn sitemap_xml(
    Extension(pool): Extension<PgPool>,
) -> Result<Response, PublicPostsError> {
    let posts = posts::list_all_published_posts(&pool)
        .await
        .map_err(PublicPostsError::Database)?;

    let body = render_sitemap_xml(&posts);

    Ok((
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        body,
    )
        .into_response())
}

pub async fn robots_txt() -> Response {
    let sitemap_url = absolute_url("/sitemap.xml");
    let body = format!("User-agent: *\nAllow: /\nSitemap: {sitemap_url}\n");

    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body).into_response()
}

pub async fn rss_feed(
    Extension(pool): Extension<PgPool>,
) -> Result<Response, PublicPostsError> {
    let posts = posts::list_all_published_posts(&pool)
        .await
        .map_err(PublicPostsError::Database)?;
    let body = render_rss_feed(&posts);

    Ok((
        [(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")],
        body,
    )
        .into_response())
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
    let description = if current_page > 1 {
        format!(
            "Page {current_page} of the myClawTeam Blog archive: published notes, updates, and delivery writeups."
        )
    } else {
        "A paginated archive of published notes, updates, and delivery writeups.".to_owned()
    };
    let page_title = if current_page > 1 {
        format!("Posts, page {current_page} | myClawTeam Blog")
    } else {
        "Posts | myClawTeam Blog".to_owned()
    };

    Ok(Html(render_posts_page(
        &published_posts,
        &category_names,
        current_page,
        total_pages,
        total_posts,
        &page_title,
        "Published posts",
        "myClawTeam Blog",
        &description,
        "/posts",
        None,
    )))
}

pub async fn category_index(
    Extension(pool): Extension<PgPool>,
    Path(slug): Path<String>,
    Query(query): Query<PaginationQuery>,
) -> Result<Html<String>, PublicPostsError> {
    let category =
        allowed_category(&slug).ok_or_else(|| PublicPostsError::CategoryNotFound(slug.clone()))?;
    let total_posts = posts::count_published_posts_by_category(&pool, category.slug)
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

    let published_posts =
        posts::list_published_posts_by_category(&pool, category.slug, POSTS_PER_PAGE, offset)
            .await
            .map_err(PublicPostsError::Database)?;
    let page_title = if current_page > 1 {
        format!("{} posts, page {} | myClawTeam Blog", category.name, current_page)
    } else {
        format!("{} posts | myClawTeam Blog", category.name)
    };
    let headline = format!("{} posts", category.name);
    let description = if current_page > 1 {
        format!(
            "Page {current_page} of {} posts from myClawTeam Blog. {}",
            category.name, category.description
        )
    } else {
        category.description.to_owned()
    };
    let base_path = format!("/categories/{}", category.slug);

    Ok(Html(render_posts_page(
        &published_posts,
        &category_names,
        current_page,
        total_pages,
        total_posts,
        &page_title,
        "Category filter",
        &headline,
        &description,
        &base_path,
        Some(category.slug),
    )))
}

pub async fn post_detail(
    Extension(pool): Extension<PgPool>,
    Extension(storage): Extension<ObjectStorage>,
    Path(slug): Path<String>,
) -> Result<Html<String>, PublicPostsError> {
    let post = posts::get_post_by_slug(&pool, &slug)
        .await
        .map_err(PublicPostsError::Database)?
        .filter(|post| post.status == "published")
        .ok_or_else(|| PublicPostsError::PostNotFound(slug.clone()))?;
    let category = posts::list_categories(&pool)
        .await
        .map_err(PublicPostsError::Database)?
        .into_iter()
        .find(|category| category.id == post.category_id);
    let body_html = render_rich_body(&pool, &storage, &post.body).await?;

    Ok(Html(render_post_detail_page(&post, category.as_ref(), &body_html)))
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
    title: &str,
    eyebrow: &str,
    headline: &str,
    description: &str,
    base_path: &str,
    active_category: Option<&str>,
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
    let pagination = render_pagination(current_page, total_pages, base_path);
    let category_filters = render_category_filters(active_category);
    let canonical_path = pagination_path(base_path, current_page);
    let metadata = render_metadata(Metadata {
        title,
        description,
        path: &canonical_path,
        og_type: "website",
    });

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>{title}</title>
{metadata}
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
<p class="text-kicker font-bold uppercase tracking-wide text-accent-400">{eyebrow}</p>
<h1 class="max-w-4xl text-display font-black leading-none text-foreground">{headline}</h1>
<p class="max-w-2xl text-lead leading-8 text-muted">{description}</p>
<p class="text-sm font-bold text-muted">{total_posts} published posts &middot; Page {current_page} of {total_pages}</p>
{category_filters}
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
</html>"#,
        title = escape_html(title),
        metadata = metadata,
        eyebrow = escape_html(eyebrow),
        headline = render_headline(headline),
        description = escape_html(description),
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
    let href = format!("/posts/{}", post.slug);

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
<a href="{href}" class="rounded-lg border border-white/10 px-3 py-2 text-sm font-black text-foreground transition group-hover:border-accent-400 group-hover:text-accent-400">Read</a>
</div>
</article>"#,
        category = escape_html(category),
        title = escape_html(&post.title),
        excerpt = escape_html(&excerpt),
        published_at = escape_html(&published_at),
        href = escape_html(&href),
    )
}

fn render_post_detail_page(
    post: &posts::Post,
    category: Option<&posts::Category>,
    body_html: &str,
) -> String {
    let category_name = category.map(|category| category.name.as_str()).unwrap_or("Post");
    let category_href = category
        .map(|category| format!("/categories/{}", category.slug))
        .unwrap_or_else(|| "/posts".to_owned());
    let published_at = post
        .published_at
        .map(|value| first_date_chars(&value.to_string()))
        .unwrap_or_else(|| "Published".to_owned());
    let description = excerpt_from_body(&post.body);
    let page_title = format!("{} | myClawTeam Blog", post.title);
    let canonical_path = format!("/posts/{}", post.slug);
    let metadata = render_metadata(Metadata {
        title: &page_title,
        description: &description,
        path: &canonical_path,
        og_type: "article",
    });

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>{title} | myClawTeam Blog</title>
{metadata}
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
<article class="mx-auto flex w-full max-w-4xl flex-col gap-8 px-6 py-16 sm:px-10 lg:py-20">
<div class="flex flex-wrap items-center gap-3 text-sm font-black uppercase tracking-wide text-muted">
<a href="{category_href}" class="rounded-lg border border-accent-400/40 px-3 py-2 text-accent-400 transition hover:bg-accent-500 hover:text-white">{category}</a>
<span>{published_at}</span>
</div>
<header class="flex flex-col gap-5">
<h1 class="text-display font-black leading-none text-foreground">{title}</h1>
</header>
<div class="flex flex-col gap-6 text-lg leading-8 text-muted">{body_html}</div>
<nav aria-label="Post navigation" class="border-t border-white/10 pt-6">
<a href="/posts" class="rounded-lg border border-white/10 px-3 py-2 text-sm font-black text-foreground transition hover:border-accent-400 hover:text-accent-400">All posts</a>
</nav>
</article>
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
</html>"#,
        title = escape_html(&post.title),
        metadata = metadata,
        category = escape_html(category_name),
        category_href = escape_html(&category_href),
        published_at = escape_html(&published_at),
    )
}

async fn render_rich_body(
    pool: &PgPool,
    storage: &ObjectStorage,
    body: &str,
) -> Result<String, PublicPostsError> {
    let mut sections = Vec::new();

    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(object_key) = parse_image_embed(line) {
            sections.push(render_media_embed(pool, storage, object_key, "image").await?);
        } else if let Some(object_key) = parse_video_embed(line) {
            sections.push(render_media_embed(pool, storage, object_key, "video").await?);
        } else if let Some(heading) = line.strip_prefix("### ") {
            sections.push(format!(
                r#"<h3 class="pt-4 text-2xl font-black leading-tight text-foreground">{}</h3>"#,
                escape_html(heading)
            ));
        } else if let Some(heading) = line.strip_prefix("## ") {
            sections.push(format!(
                r#"<h2 class="pt-5 text-3xl font-black leading-tight text-foreground">{}</h2>"#,
                escape_html(heading)
            ));
        } else if let Some(heading) = line.strip_prefix("# ") {
            sections.push(format!(
                r#"<h2 class="pt-5 text-3xl font-black leading-tight text-foreground">{}</h2>"#,
                escape_html(heading)
            ));
        } else {
            sections.push(format!(
                r#"<p class="text-muted">{}</p>"#,
                escape_html(line)
            ));
        }
    }

    if sections.is_empty() {
        Ok(r#"<p class="text-muted">This post has no body content yet.</p>"#.to_owned())
    } else {
        Ok(sections.join(""))
    }
}

async fn render_media_embed(
    pool: &PgPool,
    storage: &ObjectStorage,
    object_key: &str,
    expected_type: &str,
) -> Result<String, PublicPostsError> {
    let media = media::get_media_by_object_key(pool, object_key)
        .await
        .map_err(PublicPostsError::Database)?;
    let Some(media) = media else {
        return Ok(render_missing_media(object_key));
    };

    if media.media_type != expected_type {
        return Ok(render_missing_media(object_key));
    }

    let signed_url = storage
        .presigned_get_url(&media.object_key, PRESIGNED_MEDIA_TTL)
        .await
        .map_err(PublicPostsError::Storage)?;

    if media.media_type == "video" {
        Ok(format!(
            r#"<figure class="overflow-hidden rounded-lg border border-white/10 bg-surface-900"><video class="aspect-video w-full bg-black" controls preload="metadata" src="{src}"></video></figure>"#,
            src = escape_html(&signed_url),
        ))
    } else {
        Ok(format!(
            r#"<figure class="overflow-hidden rounded-lg border border-white/10 bg-surface-900"><img class="w-full object-cover" src="{src}" alt="" loading="lazy" /></figure>"#,
            src = escape_html(&signed_url),
        ))
    }
}

fn render_missing_media(object_key: &str) -> String {
    format!(
        r#"<div class="rounded-lg border border-accent-500/40 bg-accent-500/10 p-4 text-sm font-bold text-accent-300">Embedded media is unavailable: {}</div>"#,
        escape_html(object_key)
    )
}

fn parse_image_embed(line: &str) -> Option<&str> {
    parse_wrapped_token(line, "![media:", "]")
}

fn parse_video_embed(line: &str) -> Option<&str> {
    parse_wrapped_token(line, "[video:", "]")
}

fn parse_wrapped_token<'a>(line: &'a str, prefix: &str, suffix: &str) -> Option<&'a str> {
    let value = line.strip_prefix(prefix)?.strip_suffix(suffix)?.trim();
    if value.is_empty() || value.starts_with('/') {
        None
    } else {
        Some(value)
    }
}

fn render_pagination(current_page: i64, total_pages: i64, base_path: &str) -> String {
    if total_pages <= 1 {
        return String::new();
    }

    let previous = if current_page > 1 {
        pagination_link(base_path, current_page - 1, "Previous", false)
    } else {
        disabled_pagination_label("Previous")
    };
    let next = if current_page < total_pages {
        pagination_link(base_path, current_page + 1, "Next", false)
    } else {
        disabled_pagination_label("Next")
    };
    let pages = (1..=total_pages)
        .map(|page| pagination_link(base_path, page, &page.to_string(), page == current_page))
        .collect::<Vec<_>>()
        .join("");

    format!(
        r#"<nav aria-label="Pagination" class="flex flex-wrap items-center justify-between gap-3 border-t border-white/10 pt-6">
<div class="flex gap-2">{previous}{next}</div>
<div class="flex flex-wrap gap-2">{pages}</div>
</nav>"#
    )
}

fn pagination_link(base_path: &str, page: i64, label: &str, current: bool) -> String {
    let class = if current {
        "rounded-lg bg-accent-500 px-3 py-2 text-sm font-black text-white"
    } else {
        "rounded-lg border border-white/10 px-3 py-2 text-sm font-black text-foreground transition hover:border-accent-400 hover:text-accent-400"
    };
    let aria_current = if current { r#" aria-current="page""# } else { "" };

    format!(
        r#"<a href="{base_path}?page={page}" class="{class}"{aria_current}>{label}</a>"#,
        base_path = escape_html(base_path),
        label = escape_html(label),
    )
}

fn pagination_path(base_path: &str, page: i64) -> String {
    if page <= 1 {
        base_path.to_owned()
    } else {
        format!("{base_path}?page={page}")
    }
}

fn render_sitemap_xml(posts: &[posts::Post]) -> String {
    let mut urls = vec![
        sitemap_url("/", "daily", "0.9", None),
        sitemap_url("/posts", "daily", "0.8", None),
    ];

    urls.extend(CATEGORY_FILTERS.iter().map(|category| {
        let path = format!("/categories/{}", category.slug);
        sitemap_url(&path, "weekly", "0.7", None)
    }));

    urls.extend(posts.iter().map(|post| {
        let path = format!("/posts/{}", post.slug);
        let last_modified = post
            .updated_at
            .to_string()
            .chars()
            .take(10)
            .collect::<String>();

        sitemap_url(&path, "weekly", "0.8", Some(&last_modified))
    }));

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
{}
</urlset>
"#,
        urls.join("")
    )
}

fn sitemap_url(
    path: &str,
    changefreq: &str,
    priority: &str,
    last_modified: Option<&str>,
) -> String {
    let lastmod = last_modified
        .map(|value| format!("<lastmod>{}</lastmod>\n", escape_xml(value)))
        .unwrap_or_default();

    format!(
        "<url>\n<loc>{}</loc>\n{}<changefreq>{}</changefreq>\n<priority>{}</priority>\n</url>\n",
        escape_xml(&absolute_url(path)),
        lastmod,
        escape_xml(changefreq),
        escape_xml(priority),
    )
}

fn render_rss_feed(posts: &[posts::Post]) -> String {
    let items = posts
        .iter()
        .map(render_rss_item)
        .collect::<Vec<_>>()
        .join("");

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
<channel>
<title>{title}</title>
<link>{site_url}</link>
<description>{description}</description>
<language>en-us</language>
<atom:link href="{feed_url}" rel="self" type="application/rss+xml" />
{items}</channel>
</rss>
"#,
        title = escape_xml(SITE_NAME),
        site_url = escape_xml(&absolute_url("/")),
        description = escape_xml("By talking, serious delivery."),
        feed_url = escape_xml(&absolute_url("/feed.xml")),
        items = items,
    )
}

fn render_rss_item(post: &posts::Post) -> String {
    let path = format!("/posts/{}", post.slug);
    let url = absolute_url(&path);
    let description = excerpt_from_body(&post.body);
    let published_at = post
        .published_at
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| post.updated_at.to_string());

    format!(
        r#"<item>
<title>{title}</title>
<link>{url}</link>
<guid isPermaLink="true">{url}</guid>
<description>{description}</description>
<pubDate>{published_at}</pubDate>
</item>
"#,
        title = escape_xml(&post.title),
        url = escape_xml(&url),
        description = escape_xml(&description),
        published_at = escape_xml(&published_at),
    )
}

fn disabled_pagination_label(label: &str) -> String {
    format!(
        r#"<span class="rounded-lg border border-white/5 px-3 py-2 text-sm font-black text-muted opacity-50">{}</span>"#,
        escape_html(label)
    )
}

struct Metadata<'a> {
    title: &'a str,
    description: &'a str,
    path: &'a str,
    og_type: &'a str,
}

fn render_metadata(metadata: Metadata<'_>) -> String {
    let canonical_url = absolute_url(metadata.path);
    let image_url = absolute_url(OG_IMAGE_PATH);

    format!(
        r#"<meta name="description" content="{description}" />
<link rel="canonical" href="{canonical_url}" />
<meta property="og:site_name" content="{site_name}" />
<meta property="og:type" content="{og_type}" />
<meta property="og:title" content="{title}" />
<meta property="og:description" content="{description}" />
<meta property="og:url" content="{canonical_url}" />
<meta property="og:image" content="{image_url}" />
<meta property="og:image:alt" content="{site_name}" />
<meta name="twitter:card" content="summary_large_image" />
<meta name="twitter:title" content="{title}" />
<meta name="twitter:description" content="{description}" />
<meta name="twitter:image" content="{image_url}" />"#,
        title = escape_html(metadata.title),
        description = escape_html(metadata.description),
        canonical_url = escape_html(&canonical_url),
        site_name = escape_html(SITE_NAME),
        og_type = escape_html(metadata.og_type),
        image_url = escape_html(&image_url),
    )
}

fn absolute_url(path: &str) -> String {
    let normalized_path = if path.starts_with('/') {
        path.to_owned()
    } else {
        format!("/{path}")
    };
    let Ok(site_url) = std::env::var("SELF_URL") else {
        return normalized_path;
    };
    let site_url = site_url.trim_end_matches('/');
    if site_url.is_empty() {
        normalized_path
    } else {
        format!("{site_url}{normalized_path}")
    }
}

fn first_date_chars(value: &str) -> String {
    value.chars().take(10).collect()
}

fn allowed_category(slug: &str) -> Option<CategoryFilter> {
    CATEGORY_FILTERS
        .iter()
        .copied()
        .find(|category| category.slug == slug)
}

fn render_category_filters(active_category: Option<&str>) -> String {
    let all_class = category_filter_class(active_category.is_none());
    let mut links = vec![format!(
        r#"<a href="/posts" class="{all_class}">All</a>"#
    )];

    links.extend(CATEGORY_FILTERS.iter().map(|category| {
        let class = category_filter_class(active_category == Some(category.slug));
        format!(
            r#"<a href="/categories/{slug}" class="{class}">{name}</a>"#,
            slug = escape_html(category.slug),
            name = escape_html(category.name),
        )
    }));

    format!(
        r#"<nav aria-label="Category filters" class="flex flex-wrap gap-2">{}</nav>"#,
        links.join("")
    )
}

fn category_filter_class(active: bool) -> &'static str {
    if active {
        "rounded-lg bg-accent-500 px-3 py-2 text-sm font-black text-white"
    } else {
        "rounded-lg border border-white/10 px-3 py-2 text-sm font-black text-foreground transition hover:border-accent-400 hover:text-accent-400"
    }
}

fn render_headline(headline: &str) -> String {
    if headline == "myClawTeam Blog" {
        "myClawTeam <span class=\"text-accent-400\">Blog</span>".to_owned()
    } else {
        escape_html(headline)
    }
}

fn render_not_found_page(title: &str, message: &str) -> String {
    render_status_page("404", title, message, "View all posts", "/posts")
}

fn render_error_page(title: &str, message: &str) -> String {
    render_status_page("Error", title, message, "Return home", "/")
}

fn render_status_page(
    eyebrow: &str,
    title: &str,
    message: &str,
    link_label: &str,
    link_href: &str,
) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>{title} | myClawTeam Blog</title>
<link rel="stylesheet" href="/pkg/mike-t-4b46-mct-official-blog.css" />
</head>
<body>
<main class="min-h-screen bg-background px-6 py-24 text-foreground sm:px-10">
<div class="mx-auto flex w-full max-w-3xl flex-col gap-4">
<p class="text-kicker font-bold uppercase tracking-wide text-accent-400">{eyebrow}</p>
<h1 class="text-4xl font-black">{title}</h1>
<p class="text-muted">{message}</p>
<a href="{link_href}" class="w-fit rounded-lg border border-accent-400/40 px-3 py-2 text-sm font-black text-accent-400 transition hover:bg-accent-500 hover:text-white">{link_label}</a>
</div>
</main>
</body>
</html>"#,
        eyebrow = escape_html(eyebrow),
        title = escape_html(title),
        message = escape_html(message),
        link_label = escape_html(link_label),
        link_href = escape_html(link_href),
    )
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn escape_xml(value: &str) -> String {
    escape_html(value)
}
