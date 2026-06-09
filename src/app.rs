use leptos::prelude::*;
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
                        href="/admin"
                        class="rounded-lg border border-accent-400/50 px-3 py-2 text-sm font-bold text-accent-400"
                    >
                        "Admin"
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
        </section>
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
