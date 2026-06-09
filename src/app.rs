use leptos::{form::ActionForm, prelude::*};
use leptos_meta::{Meta, MetaTags, Stylesheet, Title, provide_meta_context};
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
                        <Route path=path!("/admin/login") view=LoginPage />
                        <ParentRoute path=path!("/admin") view=AdminLayout>
                            <Route path=path!("") view=AdminHome />
                            <Route path=path!("subscribers") view=SubscribersPage />
                            <Route path=path!("posts/new") view=PostEditorPage />
                            <Route path=path!("posts/edit") view=PostEditorPage />
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
    let logout = ServerAction::<AdminLogout>::new();
    let logout_pending = logout.pending();

    view! {
        <div class="flex min-h-screen flex-col bg-surface-900/70">
            <header class="border-b border-white/10 bg-background/80">
                <div class="mx-auto flex w-full max-w-6xl items-center justify-between px-6 py-4 sm:px-10">
                    <a href="/" class="text-sm font-bold text-foreground">
                        "myClawTeam Blog"
                    </a>
                    <div class="flex flex-wrap items-center justify-end gap-2">
                        <a
                            class="rounded-lg px-3 py-2 text-sm font-bold text-muted transition hover:bg-white/5 hover:text-foreground"
                            href="/admin"
                        >
                            "Dashboard"
                        </a>
                        <a
                            class="rounded-lg px-3 py-2 text-sm font-bold text-muted transition hover:bg-white/5 hover:text-foreground"
                            href="/admin/subscribers"
                        >
                            "Subscribers"
                        </a>
                        <ActionForm action=logout attr:class="m-0">
                            <button
                                class="rounded-lg border border-accent-400/50 px-3 py-2 text-sm font-bold text-accent-400 transition hover:bg-accent-500 hover:text-white disabled:cursor-not-allowed disabled:opacity-60"
                                type="submit"
                                disabled=move || logout_pending.get()
                            >
                                {move || if logout_pending.get() { "Signing out..." } else { "Sign out" }}
                            </button>
                        </ActionForm>
                    </div>
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
        <header class="sticky top-0 z-20 border-b border-white/10 bg-background/90 shadow-red-glow backdrop-blur-xl">
            <div class="mx-auto flex w-full max-w-6xl flex-wrap items-center justify-between gap-4 px-6 py-4 sm:px-10">
                <a href="/" class="group inline-flex items-center gap-3 text-base font-black text-foreground">
                    <span class="grid h-9 w-9 place-items-center rounded-lg border border-accent-400/40 bg-accent-500 text-sm text-white shadow-red-glow">
                        "M"
                    </span>
                    <span class="leading-tight">
                        "myClawTeam"
                        <span class="block text-xs font-bold uppercase tracking-wide text-accent-400">"Blog"</span>
                    </span>
                </a>
                <nav aria-label="Primary navigation" class="flex items-center gap-2 text-sm font-bold text-muted">
                    <a href="/" class="rounded-lg px-3 py-2 transition hover:bg-white/5 hover:text-foreground">"Home"</a>
                    <a href="/posts" class="rounded-lg px-3 py-2 transition hover:bg-white/5 hover:text-foreground">"Posts"</a>
                    <a href="/admin" class="rounded-lg border border-accent-400/40 px-3 py-2 text-accent-400 transition hover:bg-accent-500 hover:text-white">"Admin"</a>
                </nav>
            </div>
        </header>
    }
}

#[component]
fn SiteFooter() -> impl IntoView {
    view! {
        <footer class="border-t border-white/10 bg-background/70">
            <div class="mx-auto grid w-full max-w-6xl gap-6 px-6 py-10 text-sm text-muted sm:grid-cols-[1fr_auto] sm:items-end sm:px-10">
                <div>
                    <p class="text-base font-black text-foreground">"myClawTeam Blog"</p>
                    <p class="mt-2 max-w-xl leading-6">"By talking, serious delivery."</p>
                </div>
                <nav aria-label="Footer navigation" class="flex flex-wrap gap-3 font-bold">
                    <a href="/" class="transition hover:text-foreground">"Home"</a>
                    <a href="/posts" class="transition hover:text-foreground">"Posts"</a>
                    <a href="/admin" class="transition hover:text-accent-400">"Admin"</a>
                </nav>
            </div>
        </footer>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <Title text="myClawTeam Blog | By talking, serious delivery." />
        <Meta
            name="description"
            content="Recent posts, field notes, and delivery updates from the myClawTeam build."
        />
        <Meta property="og:site_name" content="myClawTeam Blog" />
        <Meta property="og:type" content="website" />
        <Meta property="og:title" content="myClawTeam Blog" />
        <Meta
            property="og:description"
            content="Recent posts, field notes, and delivery updates from the myClawTeam build."
        />
        <Meta property="og:url" content="/" />
        <Meta property="og:image" content="/og-card.svg" />
        <Meta property="og:image:alt" content="myClawTeam Blog" />
        <Meta name="twitter:card" content="summary_large_image" />
        <Meta name="twitter:title" content="myClawTeam Blog" />
        <Meta
            name="twitter:description"
            content="Recent posts, field notes, and delivery updates from the myClawTeam build."
        />
        <Meta name="twitter:image" content="/og-card.svg" />
        <div class="mx-auto flex w-full max-w-6xl flex-col gap-12 px-6 py-16 sm:px-10 lg:py-20">
            <section class="grid gap-8 lg:grid-cols-[minmax(0,1.1fr)_minmax(280px,0.7fr)] lg:items-end">
                <div class="flex flex-col gap-7">
                    <p class="text-kicker font-bold uppercase tracking-wide text-accent-400">
                        "By talking, serious delivery."
                    </p>
                    <h1 class="max-w-4xl text-display font-black leading-none text-foreground">
                        "myClawTeam "
                        <span class="text-accent-400">"Blog"</span>
                    </h1>
                    <p class="max-w-2xl text-lead leading-8 text-muted">
                        "Recent posts, field notes, and delivery updates from the myClawTeam build."
                    </p>
                </div>
                <aside class="rounded-lg border border-white/10 bg-surface-900 p-5 shadow-red-glow">
                    <p class="text-kicker font-bold uppercase tracking-wide text-accent-400">"Now"</p>
                    <p class="mt-3 text-2xl font-black text-foreground">"Serious delivery, written down."</p>
                    <p class="mt-3 leading-7 text-muted">"Featured notes and fresh posts surface here as soon as they are published."</p>
                </aside>
            </section>
            <section aria-labelledby="recent-posts-heading" class="flex flex-col gap-5">
                <div class="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
                    <div>
                        <p class="text-kicker font-bold uppercase tracking-wide text-accent-400">"Latest"</p>
                        <h2 id="recent-posts-heading" class="text-3xl font-black text-foreground">"Recent and featured posts"</h2>
                    </div>
                    <p data-home-posts-status class="text-sm font-bold text-muted">"Loading published posts..."</p>
                </div>
                <div data-home-posts class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                    <PostCard
                        category="Thoughts"
                        title="The shape of useful momentum"
                        excerpt="Short reflections on decisions, constraints, and work that keeps moving."
                        meta="Featured"
                        href="/posts"
                    />
                    <PostCard
                        category="Product Progress"
                        title="What shipped, what sharpened"
                        excerpt="A running record of product movement from rough cut to sturdier release."
                        meta="Build notes"
                        href="/posts"
                    />
                    <PostCard
                        category="Announcements"
                        title="Updates worth calling out"
                        excerpt="Launch notes and important changes for readers following the work."
                        meta="News"
                        href="/posts"
                    />
                </div>
            </section>
            <NewsletterSignup />
        </div>
        <script src="/homepage.js" defer></script>
    }
}

#[component]
fn NewsletterSignup() -> impl IntoView {
    let subscribe = ServerAction::<SubscribeNewsletter>::new();
    let pending = subscribe.pending();
    let result = subscribe.value();
    let has_success = move || result.get().is_some_and(|result| result.is_ok());
    let has_error = move || result.get().is_some_and(|result| result.is_err());

    view! {
        <section class="grid gap-6 rounded-lg border border-accent-500/30 bg-surface-900 p-6 shadow-red-glow lg:grid-cols-[minmax(0,0.8fr)_minmax(320px,1fr)] lg:items-center">
            <div>
                <p class="text-kicker font-bold uppercase tracking-wide text-accent-400">"Stay in the Loop"</p>
                <h2 class="mt-3 text-3xl font-black text-foreground">"Get the next delivery note."</h2>
                <p class="mt-3 max-w-xl leading-7 text-muted">
                    "Join the myClawTeam Blog list for new posts and product updates. No outbound email is sent yet; this just records your signup."
                </p>
            </div>
            <ActionForm action=subscribe attr:class="flex flex-col gap-3">
                <div class="flex flex-col gap-3 sm:flex-row">
                    <label class="sr-only" for="newsletter-email">"Email address"</label>
                    <input
                        id="newsletter-email"
                        class="min-h-12 flex-1 rounded-lg border border-white/10 bg-background px-4 text-base text-foreground outline-none transition placeholder:text-muted focus:border-accent-400"
                        type="email"
                        name="email"
                        placeholder="you@example.com"
                        autocomplete="email"
                        required
                        maxlength="320"
                    />
                    <button
                        class="min-h-12 rounded-lg bg-accent-500 px-5 text-sm font-black text-white transition hover:bg-accent-400 disabled:cursor-not-allowed disabled:opacity-60"
                        type="submit"
                        disabled=move || pending.get()
                    >
                        {move || if pending.get() { "Signing up..." } else { "Sign up" }}
                    </button>
                </div>
                <Show when=has_success fallback=|| ()>
                    <p class="rounded-lg border border-white/10 bg-background/70 px-3 py-2 text-sm font-bold text-foreground">
                        "You're on the list."
                    </p>
                </Show>
                <Show when=has_error fallback=|| ()>
                    <p class="rounded-lg border border-accent-500/40 bg-accent-500/10 px-3 py-2 text-sm font-bold text-accent-300">
                        "Enter a valid email address and try again."
                    </p>
                </Show>
            </ActionForm>
        </section>
    }
}

#[component]
fn PostCard(
    category: &'static str,
    title: &'static str,
    excerpt: &'static str,
    meta: &'static str,
    href: &'static str,
) -> impl IntoView {
    view! {
        <article class="group flex min-h-72 flex-col justify-between rounded-lg border border-white/10 bg-surface-900 p-5 transition hover:-translate-y-1 hover:border-accent-400/60 hover:shadow-red-glow">
            <div>
                <div class="flex items-center justify-between gap-3">
                    <p class="text-kicker font-black uppercase tracking-wide text-accent-400">{category}</p>
                    <span class="h-2 w-2 rounded-full bg-accent-500"></span>
                </div>
                <h2 class="mt-5 text-2xl font-black leading-tight text-foreground">{title}</h2>
                <p class="mt-4 leading-7 text-muted">{excerpt}</p>
            </div>
            <div class="mt-8 flex items-center justify-between gap-4 border-t border-white/10 pt-4">
                <p class="text-xs font-bold uppercase tracking-wide text-muted">{meta}</p>
                <a href=href class="rounded-lg border border-white/10 px-3 py-2 text-sm font-black text-foreground transition group-hover:border-accent-400 group-hover:text-accent-400">
                    "Read"
                </a>
            </div>
        </article>
    }
}

#[component]
fn AdminHome() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-8">
            <section class="rounded-lg border border-white/10 bg-background/70 p-6">
                <p class="text-kicker font-bold uppercase tracking-wide text-accent-400">"Admin"</p>
                <h1 class="mt-3 text-4xl font-black text-foreground">"Publishing workspace"</h1>
                <div class="mt-5 flex flex-wrap gap-3">
                    <a
                        class="inline-flex items-center justify-center rounded-lg border border-white/10 px-4 py-3 text-sm font-bold text-foreground transition hover:border-accent-400 hover:text-accent-400"
                        href="/admin/subscribers"
                    >
                        "View subscribers"
                    </a>
                </div>
            </section>
            <PostDashboard />
            <MediaPicker />
        </div>
    }
}

#[component]
fn PostDashboard() -> impl IntoView {
    view! {
        <section data-admin-posts class="rounded-lg border border-white/10 bg-background/70 p-6">
            <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div>
                    <p class="text-kicker font-bold uppercase tracking-wide text-accent-400">"Posts"</p>
                    <h2 class="mt-2 text-2xl font-black text-foreground">"Dashboard"</h2>
                </div>
                <a
                    class="inline-flex items-center justify-center rounded-lg bg-accent-500 px-4 py-3 text-sm font-black text-white transition hover:bg-accent-400"
                    href="/admin/posts/new"
                >
                    "New post"
                </a>
            </div>
            <p data-admin-posts-error class="mt-4 hidden rounded-lg border border-accent-500/40 bg-accent-500/10 px-3 py-2 text-sm font-bold text-accent-300"></p>
            <div class="mt-6 overflow-hidden rounded-lg border border-white/10 bg-surface-900">
                <div class="overflow-x-auto">
                    <table class="min-w-full border-collapse text-left text-sm">
                        <thead class="border-b border-white/10 bg-background/80 text-xs font-black uppercase tracking-wide text-muted">
                            <tr>
                                <th class="px-4 py-3">"Title"</th>
                                <th class="px-4 py-3">"Status"</th>
                                <th class="px-4 py-3">"Category"</th>
                                <th class="px-4 py-3">"Updated"</th>
                                <th class="px-4 py-3 text-right">"Actions"</th>
                            </tr>
                        </thead>
                        <tbody data-admin-posts-table class="divide-y divide-white/10">
                            <tr>
                                <td class="px-4 py-4 text-sm font-bold text-muted" colspan="5">
                                    "Loading posts..."
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
            <script src="/admin-posts.js" defer></script>
        </section>
    }
}

#[component]
fn SubscribersPage() -> impl IntoView {
    view! {
        <section data-admin-subscribers class="rounded-lg border border-white/10 bg-background/70 p-6">
            <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div>
                    <p class="text-kicker font-bold uppercase tracking-wide text-accent-400">"Subscribers"</p>
                    <h1 class="mt-2 text-3xl font-black text-foreground">"Newsletter signups"</h1>
                </div>
                <a
                    class="inline-flex items-center justify-center rounded-lg border border-white/10 px-4 py-3 text-sm font-bold text-foreground transition hover:border-accent-400 hover:text-accent-400"
                    href="/admin"
                >
                    "Back to dashboard"
                </a>
            </div>
            <p class="mt-3 max-w-2xl leading-7 text-muted">
                "Collected email addresses from the Stay in the Loop form."
            </p>
            <p data-admin-subscribers-error class="mt-4 hidden rounded-lg border border-accent-500/40 bg-accent-500/10 px-3 py-2 text-sm font-bold text-accent-300"></p>
            <div class="mt-6 overflow-hidden rounded-lg border border-white/10 bg-surface-900">
                <div class="overflow-x-auto">
                    <table class="min-w-full border-collapse text-left text-sm">
                        <thead class="border-b border-white/10 bg-background/80 text-xs font-black uppercase tracking-wide text-muted">
                            <tr>
                                <th class="px-4 py-3">"Email"</th>
                                <th class="px-4 py-3">"Signed up"</th>
                                <th class="px-4 py-3 text-right">"Subscriber ID"</th>
                            </tr>
                        </thead>
                        <tbody data-admin-subscribers-table class="divide-y divide-white/10">
                            <tr>
                                <td class="px-4 py-4 text-sm font-bold text-muted" colspan="3">
                                    "Loading subscribers..."
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
            <script src="/admin-subscribers.js" defer></script>
        </section>
    }
}

#[component]
fn PostEditorPage() -> impl IntoView {
    view! {
        <section data-post-editor class="rounded-lg border border-white/10 bg-background/70 p-6">
            <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div>
                    <p class="text-kicker font-bold uppercase tracking-wide text-accent-400">"Posts"</p>
                    <h1 data-post-editor-heading class="mt-2 text-3xl font-black text-foreground">"New post"</h1>
                </div>
                <a
                    class="inline-flex items-center justify-center rounded-lg border border-white/10 px-4 py-3 text-sm font-bold text-foreground transition hover:border-accent-400 hover:text-accent-400"
                    href="/admin"
                >
                    "Back"
                </a>
            </div>

            <p data-post-editor-error class="mt-4 hidden rounded-lg border border-accent-500/40 bg-accent-500/10 px-3 py-2 text-sm font-bold text-accent-300"></p>
            <p data-post-editor-status class="mt-4 hidden rounded-lg border border-emerald-400/40 bg-emerald-500/10 px-3 py-2 text-sm font-bold text-emerald-300"></p>

            <div class="mt-6 grid gap-6 xl:grid-cols-[minmax(0,1fr)_360px]">
                <form data-post-editor-form class="flex flex-col gap-5 rounded-lg border border-white/10 bg-surface-900 p-5">
                    <label class="flex flex-col gap-2 text-sm font-bold text-foreground">
                        "Title"
                        <input
                            data-post-title
                            class="rounded-lg border border-white/10 bg-background px-3 py-3 text-base text-foreground outline-none transition placeholder:text-muted focus:border-accent-400"
                            type="text"
                            name="title"
                            maxlength="220"
                            required
                        />
                    </label>

                    <div class="grid gap-4 sm:grid-cols-2">
                        <label class="flex flex-col gap-2 text-sm font-bold text-foreground">
                            "Category"
                            <select
                                data-post-category
                                class="rounded-lg border border-white/10 bg-background px-3 py-3 text-base text-foreground outline-none transition focus:border-accent-400"
                                name="category_id"
                                required
                            >
                                <option value="">"Loading..."</option>
                            </select>
                        </label>
                        <label class="flex flex-col gap-2 text-sm font-bold text-foreground">
                            "Status"
                            <select
                                data-post-status
                                class="rounded-lg border border-white/10 bg-background px-3 py-3 text-base text-foreground outline-none transition focus:border-accent-400"
                                name="status"
                            >
                                <option value="draft">"Draft"</option>
                                <option value="published">"Published"</option>
                            </select>
                        </label>
                    </div>

                    <label class="flex flex-col gap-2 text-sm font-bold text-foreground">
                        "Body"
                        <textarea
                            data-post-body
                            class="min-h-96 resize-y rounded-lg border border-white/10 bg-background px-3 py-3 font-mono text-sm font-normal leading-6 text-foreground outline-none transition focus:border-accent-400"
                            name="body"
                        ></textarea>
                    </label>

                    <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-end">
                        <button
                            class="rounded-lg bg-accent-500 px-4 py-3 text-sm font-black text-white transition hover:bg-accent-400 disabled:cursor-not-allowed disabled:opacity-60"
                            type="submit"
                        >
                            "Save"
                        </button>
                    </div>
                </form>

                <aside class="flex flex-col gap-5 rounded-lg border border-white/10 bg-surface-900 p-5">
                    <div>
                        <p class="text-kicker font-bold uppercase tracking-wide text-accent-400">"Media"</p>
                        <h2 class="mt-2 text-xl font-black text-foreground">"Embed"</h2>
                    </div>
                    <form data-post-media-upload class="flex flex-col gap-3" enctype="multipart/form-data">
                        <input
                            class="rounded-lg border border-white/10 bg-background px-3 py-3 text-sm text-foreground file:mr-3 file:rounded-md file:border-0 file:bg-accent-500 file:px-3 file:py-2 file:text-sm file:font-black file:text-white"
                            type="file"
                            name="file"
                            accept="image/jpeg,image/png,image/gif,image/webp,image/avif,video/mp4,video/webm,video/quicktime"
                            required
                        />
                        <button
                            class="rounded-lg border border-accent-400/50 px-3 py-2 text-sm font-black text-accent-400 transition hover:bg-accent-500 hover:text-white disabled:cursor-not-allowed disabled:opacity-60"
                            type="submit"
                        >
                            "Upload"
                        </button>
                    </form>
                    <div data-post-media-grid class="grid grid-cols-1 gap-3">
                        <div class="rounded-lg border border-white/10 bg-background p-4 text-sm font-bold text-muted">
                            "Loading media..."
                        </div>
                    </div>
                </aside>
            </div>
            <script src="/post-editor.js" defer></script>
        </section>
    }
}

#[component]
fn MediaPicker() -> impl IntoView {
    view! {
        <section data-media-picker class="rounded-lg border border-white/10 bg-background/70 p-6">
            <div class="flex flex-col gap-6 lg:flex-row lg:items-start lg:justify-between">
                <div>
                    <p class="text-kicker font-bold uppercase tracking-wide text-accent-400">"Media"</p>
                    <h2 class="mt-2 text-2xl font-black text-foreground">"Library"</h2>
                </div>
                <form
                    data-media-upload-form
                    class="flex w-full flex-col gap-3 rounded-lg border border-white/10 bg-surface-900 p-4 lg:max-w-sm"
                    enctype="multipart/form-data"
                >
                    <label class="flex flex-col gap-2 text-sm font-bold text-foreground">
                        "Upload"
                        <input
                            class="rounded-lg border border-white/10 bg-background px-3 py-3 text-sm text-foreground file:mr-3 file:rounded-md file:border-0 file:bg-accent-500 file:px-3 file:py-2 file:text-sm file:font-black file:text-white"
                            type="file"
                            name="file"
                            accept="image/jpeg,image/png,image/gif,image/webp,image/avif,video/mp4,video/webm,video/quicktime"
                            required
                        />
                    </label>
                    <button
                        class="rounded-lg bg-accent-500 px-4 py-3 text-sm font-black text-white transition hover:bg-accent-400 disabled:cursor-not-allowed disabled:opacity-60"
                        type="submit"
                    >
                        "Upload"
                    </button>
                    <p data-media-error class="hidden rounded-lg border border-accent-500/40 bg-accent-500/10 px-3 py-2 text-sm font-bold text-accent-300"></p>
                </form>
            </div>

            <div class="mt-6 grid gap-6 lg:grid-cols-[minmax(0,1fr)_320px]">
                <div data-media-grid class="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
                    <div class="rounded-lg border border-white/10 bg-surface-900 p-4 text-sm font-bold text-muted">
                        "Loading media..."
                    </div>
                </div>
                <aside class="rounded-lg border border-white/10 bg-surface-900 p-4">
                    <label class="flex flex-col gap-3 text-sm font-bold text-foreground">
                        "Selected embed"
                        <textarea
                            data-media-selected
                            class="min-h-32 resize-y rounded-lg border border-white/10 bg-background px-3 py-3 font-mono text-sm font-normal text-foreground outline-none transition focus:border-accent-400"
                            readonly
                        ></textarea>
                    </label>
                </aside>
            </div>
            <script src="/media-picker.js" defer></script>
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

#[server]
async fn admin_logout() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::{HeaderValue, header};
        use leptos_axum::{ResponseOptions, redirect};

        use crate::auth;

        let cookie = HeaderValue::from_str(&auth::clear_admin_session_cookie()).map_err(|_| {
            ServerFnError::ServerError("Could not clear the admin session cookie.".to_owned())
        })?;
        let response = use_context::<ResponseOptions>().ok_or_else(|| {
            ServerFnError::ServerError("Response options are unavailable.".to_owned())
        })?;

        response.append_header(header::SET_COOKIE, cookie);
        redirect("/admin/login");

        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError(
            "Admin logout is only available on the server.".to_owned(),
        ))
    }
}

#[server]
async fn subscribe_newsletter(email: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::repositories::subscribers;

        let email = validate_subscriber_email(email)?;
        let pool = use_context::<sqlx::PgPool>().ok_or_else(|| {
            ServerFnError::ServerError("Database pool is unavailable.".to_owned())
        })?;

        subscribers::create_subscriber(&pool, &email)
            .await
            .map_err(|error| {
                eprintln!("failed to create newsletter subscriber: {error}");
                ServerFnError::ServerError("Could not save newsletter signup.".to_owned())
            })?;

        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = email;
        Err(ServerFnError::ServerError(
            "Newsletter signup is only available on the server.".to_owned(),
        ))
    }
}

fn validate_subscriber_email(email: String) -> Result<String, ServerFnError> {
    let email = email.trim().to_ascii_lowercase();
    if email.is_empty() {
        return Err(ServerFnError::ServerError(
            "Email address is required.".to_owned(),
        ));
    }
    if email.len() > 320 || email.chars().any(char::is_whitespace) {
        return Err(ServerFnError::ServerError(
            "Enter a valid email address.".to_owned(),
        ));
    }

    if email.matches('@').count() != 1 {
        return Err(ServerFnError::ServerError(
            "Enter a valid email address.".to_owned(),
        ));
    }

    let Some((local, domain)) = email.split_once('@') else {
        return Err(ServerFnError::ServerError(
            "Enter a valid email address.".to_owned(),
        ));
    };

    if local.is_empty()
        || domain.is_empty()
        || !domain.contains('.')
        || domain.starts_with('.')
        || domain.ends_with('.')
    {
        return Err(ServerFnError::ServerError(
            "Enter a valid email address.".to_owned(),
        ));
    }

    Ok(email)
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
