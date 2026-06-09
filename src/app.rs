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
            <main class="app-shell">
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
        <section class="hero">
            <p class="eyebrow">"SSR + hydrating client bundle"</p>
            <h1>"myClawTeam Blog"</h1>
            <p class="slogan">"By talking, serious delivery."</p>
            <button type="button" on:click=increment>
                "Hydration check: " {count}
            </button>
        </section>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <section class="not-found">
            <h1>"Page not found"</h1>
            <p>"The requested page does not exist."</p>
        </section>
    }
}
