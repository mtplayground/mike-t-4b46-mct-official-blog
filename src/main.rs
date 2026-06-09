#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use axum::{
        Extension, Router,
        extract::DefaultBodyLimit,
        middleware,
        routing::{get, post, put},
    };
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use mike_t_4b46_mct_official_blog::{
        app::{App, shell},
        admin_posts,
        admin_subscribers,
        auth,
        config::AppConfig,
        db,
        public_posts,
        storage::ObjectStorage,
        uploads,
    };

    let app_config = AppConfig::from_env()?;
    let conf = get_configuration(None)?;
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);
    let db_pool = db::connect(&app_config.database).await?;
    let object_storage = ObjectStorage::new(&app_config.object_storage);

    db::run_migrations(&db_pool).await?;
    db::verify_connectivity(&db_pool).await?;
    log!("database connection verified");
    log!("object storage client configured");

    let provide_db_context = {
        let db_pool = db_pool.clone();
        let app_config = app_config.clone();
        let object_storage = object_storage.clone();
        move || {
            provide_context(db_pool.clone());
            provide_context(app_config.clone());
            provide_context(object_storage.clone());
        }
    };
    let render_shell = {
        let leptos_options = leptos_options.clone();
        move || shell(leptos_options.clone())
    };

    let app = Router::new()
        .route("/posts", get(public_posts::posts_index))
        .route("/posts/{slug}", get(public_posts::post_detail))
        .route("/categories/{slug}", get(public_posts::category_index))
        .route("/sitemap.xml", get(public_posts::sitemap_xml))
        .route("/robots.txt", get(public_posts::robots_txt))
        .route("/api/posts/recent", get(public_posts::recent_posts))
        .route("/admin/api/categories", get(admin_posts::list_categories))
        .route(
            "/admin/api/subscribers",
            get(admin_subscribers::list_subscribers),
        )
        .route("/admin/api/media", get(uploads::list_media))
        .route(
            "/admin/api/media/upload",
            post(uploads::upload_media)
                .layer(DefaultBodyLimit::max(uploads::MAX_MULTIPART_BYTES)),
        )
        .route(
            "/admin/api/posts",
            get(admin_posts::list_posts).post(admin_posts::create_post),
        )
        .route(
            "/admin/api/posts/{id}",
            get(admin_posts::get_post)
                .put(admin_posts::update_post)
                .delete(admin_posts::delete_post),
        )
        .route(
            "/admin/api/posts/{id}/status",
            put(admin_posts::set_post_status),
        )
        .leptos_routes_with_context(&leptos_options, routes, provide_db_context, render_shell)
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(Extension(db_pool.clone()))
        .layer(Extension(object_storage.clone()))
        .layer(middleware::from_fn_with_state(
            app_config.clone(),
            auth::require_admin_session,
        ))
        .with_state(leptos_options);

    log!("listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
