use leptos::{form::ActionForm, prelude::*};
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Outlet, ParentRoute, Route, Router, Routes},
    path,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/mike-t-4b46-mct-official-blog.css" />
        <Title text="myClawTeam Blog" />
        <Router>
            <RootLayout>
                <ErrorBoundary fallback=|_| view! { <AppErrorBoundary /> }>
                    <Routes fallback=|| view! { <NotFound /> }.into_view()>
                        <ParentRoute path=path!("/") view=PublicLayout>
                            <Route path=path!("") view=HomePage />
                        </ParentRoute>
                        <ParentRoute path=path!("/admin") view=AdminLayout>
                            <Route path=path!("") view=AdminHome />
                            <Route path=path!("login") view=LoginPage />
                        </ParentRoute>
                    </Routes>
                </ErrorBoundary>
            </RootLayout>
        </Router>
    }
}

#[component]
fn RootLayout(children: Children) -> impl IntoView {
    view! {
        <div class="min-h-screen bg-background text-foreground antialiased">
            {children()}
        </div>
    }
}

#[component]
fn PublicLayout() -> impl IntoView {
    view! {
        <div class="flex min-h-screen flex-col">
            <SiteHeader />
            <main id="content" class="flex-1">
                <Outlet />
            </main>
            <SiteFooter />
        </div>
    }
}

#[component]
fn AdminLayout() -> impl IntoView {
    view! {
        <div class="flex min-h-screen flex-col bg-surface-900/70">
            <header class="border-b border-white/10 bg-background/80">
                <div class="mx-auto flex w-full max-w-6xl items-center justify-between px-6 py-4 sm:px-10">
                    <a href="/" class="text-sm font-bold text-foreground">
                        "myClawTeam Blog"
                    </a>
                    <a
                        href="/admin/login"
                        class="rounded-lg border border-accent-400/50 px-3 py-2 text-sm font-bold text-accent-400"
                    >
                        "Sign in"
                    </a>
                </div>
            </header>
            <main class="mx-auto w-full max-w-6xl flex-1 px-6 py-10 sm:px-10">
                <Outlet />
            </main>
        </div>
    }
}

#[component]
fn SiteHeader() -> impl IntoView {
    view! {
        <header class="sticky top-0 z-10 border-b border-white/10 bg-background/85 backdrop-blur">
            <div class="mx-auto flex w-full max-w-6xl flex-wrap items-center justify-between gap-4 px-6 py-4 sm:px-10">
                <a href="/" class="text-base font-black text-foreground">
                    "myClawTeam Blog"
                </a>
                <nav aria-label="Primary navigation" class="flex items-center gap-4 text-sm font-bold text-muted">
                    <a href="/" class="transition hover:text-foreground">"Home"</a>
                    <a href="/admin" class="transition hover:text-accent-400">"Admin"</a>
                </nav>
            </div>
        </header>
    }
}

#[component]
fn SiteFooter() -> impl IntoView {
    view! {
        <footer class="border-t border-white/10">
            <div class="mx-auto flex w-full max-w-6xl flex-col gap-2 px-6 py-8 text-sm text-muted sm:flex-row sm:items-center sm:justify-between sm:px-10">
                <p>"myClawTeam Blog"</p>
                <p>"By talking, serious delivery."</p>
            </div>
        </footer>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <section class="mx-auto flex w-full max-w-6xl flex-col gap-8 px-6 py-24 sm:px-10">
            <p class="text-kicker font-bold uppercase tracking-wide text-accent-400">
                "By talking, serious delivery."
            </p>
            <h1 class="max-w-4xl text-display font-black leading-none text-foreground">
                "myClawTeam Blog"
            </h1>
            <p class="max-w-2xl text-lead leading-8 text-muted">
                "Notes, progress, and announcements from the myClawTeam build."
            </p>
        </section>
    }
}

#[component]
fn AdminHome() -> impl IntoView {
    view! {
        <section class="rounded-lg border border-white/10 bg-background/70 p-6">
            <p class="text-kicker font-bold uppercase tracking-wide text-accent-400">"Admin"</p>
            <h1 class="mt-3 text-4xl font-black text-foreground">"Publishing workspace"</h1>
            <a
                href="/admin/login"
                class="mt-6 inline-flex rounded-lg bg-accent-500 px-4 py-3 text-sm font-black text-white transition hover:bg-accent-400"
            >
                "Sign in"
            </a>
        </section>
    }
}

#[component]
fn LoginPage() -> impl IntoView {
    let login = ServerAction::<AdminLogin>::new();
    let pending = login.pending();
    let value = login.value();
    let has_error = move || matches!(value.get(), Some(Err(_)));

    view! {
        <section class="mx-auto flex w-full max-w-md flex-col gap-6 rounded-lg border border-white/10 bg-background/80 p-6 shadow-2xl shadow-black/20">
            <div>
                <p class="text-kicker font-bold uppercase tracking-wide text-accent-400">"Admin"</p>
                <h1 class="mt-3 text-3xl font-black text-foreground">"Sign in"</h1>
            </div>
            <ActionForm action=login attr:class="flex flex-col gap-4">
                <label class="flex flex-col gap-2 text-sm font-bold text-foreground">
                    "Username"
                    <input
                        class="rounded-lg border border-white/10 bg-surface-900 px-3 py-3 text-base text-foreground outline-none transition placeholder:text-muted focus:border-accent-400"
                        type="text"
                        name="username"
                        autocomplete="username"
                        required
                    />
                </label>
                <label class="flex flex-col gap-2 text-sm font-bold text-foreground">
                    "Password"
                    <input
                        class="rounded-lg border border-white/10 bg-surface-900 px-3 py-3 text-base text-foreground outline-none transition placeholder:text-muted focus:border-accent-400"
                        type="password"
                        name="password"
                        autocomplete="current-password"
                        required
                    />
                </label>
                <Show when=has_error fallback=|| ()>
                    <p class="rounded-lg border border-accent-500/40 bg-accent-500/10 px-3 py-2 text-sm font-bold text-accent-300">
                        "Invalid username or password."
                    </p>
                </Show>
                <button
                    class="rounded-lg bg-accent-500 px-4 py-3 text-sm font-black text-white transition hover:bg-accent-400 disabled:cursor-not-allowed disabled:opacity-60"
                    type="submit"
                    disabled=move || pending.get()
                >
                    {move || if pending.get() { "Signing in..." } else { "Sign in" }}
                </button>
            </ActionForm>
        </section>
    }
}

#[server]
async fn admin_login(username: String, password: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::{HeaderValue, header};
        use leptos_axum::{ResponseOptions, redirect};

        use crate::{auth, config::AppConfig};

        let config = use_context::<AppConfig>().ok_or_else(|| {
            ServerFnError::ServerError("Application configuration is unavailable.".to_owned())
        })?;
        let cookie =
            auth::authenticate_admin_cookie(&config.admin, &config.session, &username, &password)
                .map_err(|_| {
                    ServerFnError::ServerError("Invalid username or password.".to_owned())
                })?;
        let cookie = HeaderValue::from_str(&cookie).map_err(|_| {
            ServerFnError::ServerError("Could not set the admin session cookie.".to_owned())
        })?;
        let response = use_context::<ResponseOptions>().ok_or_else(|| {
            ServerFnError::ServerError("Response options are unavailable.".to_owned())
        })?;

        response.append_header(header::SET_COOKIE, cookie);
        redirect("/admin");

        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (username, password);
        Err(ServerFnError::ServerError(
            "Admin login is only available on the server.".to_owned(),
        ))
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="flex min-h-screen flex-col">
            <SiteHeader />
            <main class="mx-auto flex w-full max-w-3xl flex-1 flex-col justify-center gap-4 px-6 py-24 sm:px-10">
                <p class="text-kicker font-bold uppercase tracking-wide text-accent-400">"404"</p>
                <h1 class="text-4xl font-black text-foreground">"Page not found"</h1>
                <p class="text-muted">"The requested page does not exist."</p>
            </main>
            <SiteFooter />
        </div>
    }
}

#[component]
fn AppErrorBoundary() -> impl IntoView {
    view! {
        <div class="flex min-h-screen flex-col">
            <SiteHeader />
            <main class="mx-auto flex w-full max-w-3xl flex-1 flex-col justify-center gap-4 px-6 py-24 sm:px-10">
                <p class="text-kicker font-bold uppercase tracking-wide text-accent-400">"Error"</p>
                <h1 class="text-4xl font-black text-foreground">"Something went wrong"</h1>
                <p class="text-muted">"The page could not be rendered."</p>
            </main>
            <SiteFooter />
        </div>
    }
}
