(() => {
  const root = document.querySelector("[data-admin-posts]");
  if (!root || root.dataset.adminPostsReady === "true") {
    return;
  }
  root.dataset.adminPostsReady = "true";

  const table = root.querySelector("[data-admin-posts-table]");
  const errorBox = root.querySelector("[data-admin-posts-error]");
  if (!table) {
    return;
  }

  const setError = (message) => {
    if (!errorBox) {
      return;
    }

    errorBox.textContent = message || "";
    errorBox.classList.toggle("hidden", !message);
  };

  const formatDate = (value) => {
    if (!value) {
      return "";
    }

    const date = new Date(value);
    if (Number.isNaN(date.getTime())) {
      return value;
    }

    return date.toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  };

  const statusClass = (status) => {
    if (status === "published") {
      return "border-emerald-400/40 bg-emerald-500/10 text-emerald-300";
    }

    return "border-accent-400/40 bg-accent-500/10 text-accent-300";
  };

  const createCell = (className) => {
    const cell = document.createElement("td");
    cell.className = className;
    return cell;
  };

  const renderEmpty = () => {
    const row = document.createElement("tr");
    const cell = createCell("px-4 py-6 text-sm font-bold text-muted");
    cell.colSpan = 5;
    cell.textContent = "No posts yet.";
    row.append(cell);
    table.replaceChildren(row);
  };

  const renderPosts = (posts) => {
    if (!posts.length) {
      renderEmpty();
      return;
    }

    const rows = posts.map((post) => {
      const row = document.createElement("tr");
      row.className = "align-top";

      const title = createCell("px-4 py-4");
      const titleText = document.createElement("p");
      titleText.className = "font-black text-foreground";
      titleText.textContent = post.title;
      const slug = document.createElement("p");
      slug.className = "mt-1 break-all font-mono text-xs text-muted";
      slug.textContent = post.slug;
      title.append(titleText, slug);

      const status = createCell("px-4 py-4");
      const badge = document.createElement("span");
      badge.className = `inline-flex rounded-full border px-2.5 py-1 text-xs font-black uppercase tracking-wide ${statusClass(post.status)}`;
      badge.textContent = post.status;
      status.append(badge);

      const category = createCell("px-4 py-4 font-bold text-foreground");
      category.textContent = post.category_name;

      const updated = createCell("px-4 py-4 text-muted");
      updated.textContent = formatDate(post.updated_at);

      const actions = createCell("px-4 py-4");
      const actionGroup = document.createElement("div");
      actionGroup.className = "flex justify-end gap-2";

      const edit = document.createElement("a");
      edit.className = "rounded-lg border border-white/10 px-3 py-2 text-sm font-bold text-foreground transition hover:border-accent-400 hover:text-accent-400";
      edit.href = `/admin/posts/edit?id=${post.id}`;
      edit.textContent = "Edit";

      const remove = document.createElement("button");
      remove.type = "button";
      remove.className = "rounded-lg border border-accent-400/50 px-3 py-2 text-sm font-bold text-accent-400 transition hover:bg-accent-500 hover:text-white disabled:cursor-not-allowed disabled:opacity-60";
      remove.textContent = "Delete";
      remove.addEventListener("click", async () => {
        if (!window.confirm(`Delete "${post.title}"?`)) {
          return;
        }

        remove.disabled = true;
        setError("");

        try {
          const response = await fetch(`/admin/api/posts/${post.id}`, {
            method: "DELETE",
            credentials: "same-origin",
          });

          if (!response.ok) {
            let message = "Could not delete post.";
            try {
              const payload = await response.json();
              message = payload.error || message;
            } catch (_error) {
            }
            throw new Error(message);
          }

          await loadPosts();
        } catch (error) {
          remove.disabled = false;
          setError(error instanceof Error ? error.message : "Could not delete post.");
        }
      });

      actionGroup.append(edit, remove);
      actions.append(actionGroup);
      row.append(title, status, category, updated, actions);

      return row;
    });

    table.replaceChildren(...rows);
  };

  const loadPosts = async () => {
    setError("");
    const response = await fetch("/admin/api/posts", { credentials: "same-origin" });
    if (!response.ok) {
      throw new Error("Could not load posts.");
    }

    renderPosts(await response.json());
  };

  loadPosts().catch((error) => {
    setError(error instanceof Error ? error.message : "Could not load posts.");
    renderEmpty();
  });
})();
