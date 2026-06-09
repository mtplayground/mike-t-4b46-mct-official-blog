use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
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
            <main class="app-shell min-h-screen bg-background text-foreground antialiased">
                <Routes fallback=|| view! { <NotFound /> }.into_view()>
                    <Route path=StaticSegment("") view=HomePage />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let count = RwSignal::new(0_u32);
    let increment = move |_| {
        count.update(|value| *value += 1);
    };

    view! {
        <section class="mx-auto flex w-full max-w-5xl flex-col gap-8 px-6 py-24 sm:px-10">
            <p class="text-kicker font-bold uppercase tracking-wide text-accent-400">
                "SSR + hydrating client bundle"
            </p>
            <h1 class="max-w-4xl text-display font-black leading-none text-foreground">
                "myClawTeam Blog"
            </h1>
            <p class="max-w-2xl text-lead leading-8 text-muted">
                "By talking, serious delivery."
            </p>
            <button
                type="button"
                class="w-fit rounded-lg border border-accent-400 bg-accent-600 px-5 py-3 text-sm font-bold text-white shadow-red-glow transition hover:bg-accent-500 focus:outline-none focus:ring-4 focus:ring-accent-500/40"
                on:click=increment
            >
                "Hydration check: " {count}
            </button>
        </section>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <section class="mx-auto flex min-h-screen w-full max-w-3xl flex-col justify-center gap-4 px-6 py-24">
            <h1 class="text-4xl font-black text-foreground">"Page not found"</h1>
            <p class="text-muted">"The requested page does not exist."</p>
        </section>
    }
}
