(function () {
  const grid = document.querySelector("[data-home-posts]");
  const status = document.querySelector("[data-home-posts-status]");

  if (!grid) {
    return;
  }

  const setStatus = (message) => {
    if (status) {
      status.textContent = message;
    }
  };

  const formatDate = (timestamp) => {
    if (!timestamp) {
      return "Featured";
    }

    try {
      return new Intl.DateTimeFormat(undefined, {
        month: "short",
        day: "numeric",
        year: "numeric",
      }).format(new Date(timestamp * 1000));
    } catch (_error) {
      return "Published";
    }
  };

  const createCard = (post) => {
    const article = document.createElement("article");
    article.className =
      "group flex min-h-72 flex-col justify-between rounded-lg border border-white/10 bg-surface-900 p-5 transition hover:-translate-y-1 hover:border-accent-400/60 hover:shadow-red-glow";

    const body = document.createElement("div");

    const cardHeader = document.createElement("div");
    cardHeader.className = "flex items-center justify-between gap-3";

    const category = document.createElement("p");
    category.className =
      "text-kicker font-black uppercase tracking-wide text-accent-400";
    category.textContent = post.category || "Post";

    const dot = document.createElement("span");
    dot.className = "h-2 w-2 rounded-full bg-accent-500";

    cardHeader.append(category, dot);

    const title = document.createElement("h2");
    title.className = "mt-5 text-2xl font-black leading-tight text-foreground";
    title.textContent = post.title || "Untitled post";

    const excerpt = document.createElement("p");
    excerpt.className = "mt-4 leading-7 text-muted";
    excerpt.textContent = post.excerpt || "A published note from myClawTeam Blog.";

    body.append(cardHeader, title, excerpt);

    const footer = document.createElement("div");
    footer.className =
      "mt-8 flex items-center justify-between gap-4 border-t border-white/10 pt-4";

    const meta = document.createElement("p");
    meta.className = "text-xs font-bold uppercase tracking-wide text-muted";
    meta.textContent = formatDate(post.published_at);

    const link = document.createElement("a");
    link.className =
      "rounded-lg border border-white/10 px-3 py-2 text-sm font-black text-foreground transition group-hover:border-accent-400 group-hover:text-accent-400";
    link.href = post.slug ? `/posts/${post.slug}` : "/posts";
    link.textContent = "Read";
    link.setAttribute("aria-label", `Read ${post.title || "post"}`);

    footer.append(meta, link);
    article.append(body, footer);

    return article;
  };

  fetch("/api/posts/recent", {
    headers: { Accept: "application/json" },
    credentials: "same-origin",
  })
    .then((response) => {
      if (!response.ok) {
        throw new Error(`Recent posts request failed: ${response.status}`);
      }

      return response.json();
    })
    .then((posts) => {
      if (!Array.isArray(posts) || posts.length === 0) {
        setStatus("Featured posts");
        return;
      }

      grid.replaceChildren(...posts.map(createCard));
      setStatus("Recent published posts");
    })
    .catch((error) => {
      console.error(error);
      setStatus("Featured posts");
    });
})();
