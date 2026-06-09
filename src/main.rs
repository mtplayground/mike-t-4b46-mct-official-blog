#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use mike_t_4b46_mct_official_blog::{
        app::{App, shell},
        db,
    };

    let conf = get_configuration(None)?;
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);
    let db_pool = db::connect_from_env().await?;

    db::run_migrations(&db_pool).await?;
    db::verify_connectivity(&db_pool).await?;
    log!("database connection verified");

    let provide_db_context = {
        let db_pool = db_pool.clone();
        move || {
            provide_context(db_pool.clone());
        }
    };
    let render_shell = {
        let leptos_options = leptos_options.clone();
        move || shell(leptos_options.clone())
    };

    let app = Router::new()
        .leptos_routes_with_context(&leptos_options, routes, provide_db_context, render_shell)
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    log!("listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
