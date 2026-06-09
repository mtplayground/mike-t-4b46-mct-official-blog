(() => {
  const root = document.querySelector("[data-post-editor]");
  if (!root || root.dataset.postEditorReady === "true") {
    return;
  }
  root.dataset.postEditorReady = "true";

  const form = root.querySelector("[data-post-editor-form]");
  const heading = root.querySelector("[data-post-editor-heading]");
  const titleInput = root.querySelector("[data-post-title]");
  const categorySelect = root.querySelector("[data-post-category]");
  const statusSelect = root.querySelector("[data-post-status]");
  const bodyInput = root.querySelector("[data-post-body]");
  const errorBox = root.querySelector("[data-post-editor-error]");
  const statusBox = root.querySelector("[data-post-editor-status]");
  const mediaUploadForm = root.querySelector("[data-post-media-upload]");
  const mediaGrid = root.querySelector("[data-post-media-grid]");
  if (!form || !titleInput || !categorySelect || !statusSelect || !bodyInput || !mediaGrid) {
    return;
  }

  const params = new URLSearchParams(window.location.search);
  const postId = params.get("id");
  const isEdit = Boolean(postId);

  const setMessage = (box, message) => {
    if (!box) {
      return;
    }

    box.textContent = message || "";
    box.classList.toggle("hidden", !message);
  };

  const setError = (message) => setMessage(errorBox, message);
  const setStatus = (message) => setMessage(statusBox, message);

  const formatBytes = (value) => {
    const bytes = Number(value);
    if (!Number.isFinite(bytes) || bytes < 0) {
      return "";
    }
    if (bytes >= 1024 * 1024) {
      return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
    }
    if (bytes >= 1024) {
      return `${(bytes / 1024).toFixed(1)} KiB`;
    }
    return `${bytes} B`;
  };

  const embedFor = (item) => {
    if (item.media_type === "video") {
      return `[video:${item.object_key}]`;
    }
    return `![media:${item.object_key}]`;
  };

  const insertIntoBody = (snippet) => {
    const start = bodyInput.selectionStart ?? bodyInput.value.length;
    const end = bodyInput.selectionEnd ?? bodyInput.value.length;
    const before = bodyInput.value.slice(0, start);
    const after = bodyInput.value.slice(end);
    const prefix = before && !before.endsWith("\n") ? "\n" : "";
    const suffix = after && !after.startsWith("\n") ? "\n" : "";
    const inserted = `${prefix}${snippet}${suffix}`;

    bodyInput.value = `${before}${inserted}${after}`;
    bodyInput.focus();
    const cursor = before.length + inserted.length;
    bodyInput.setSelectionRange(cursor, cursor);
  };

  const createPreview = (item) => {
    if (item.media_type === "video") {
      const video = document.createElement("video");
      video.src = item.object_url;
      video.controls = true;
      video.preload = "metadata";
      video.className = "h-full w-full object-cover";
      return video;
    }

    const image = document.createElement("img");
    image.src = item.object_url;
    image.alt = item.object_key;
    image.loading = "lazy";
    image.className = "h-full w-full object-cover";
    return image;
  };

  const renderMedia = (items) => {
    mediaGrid.replaceChildren();

    if (!items.length) {
      const empty = document.createElement("div");
      empty.className = "rounded-lg border border-white/10 bg-background p-4 text-sm font-bold text-muted";
      empty.textContent = "No media yet.";
      mediaGrid.append(empty);
      return;
    }

    for (const item of items) {
      const card = document.createElement("article");
      card.className = "overflow-hidden rounded-lg border border-white/10 bg-background";

      const frame = document.createElement("div");
      frame.className = "aspect-video bg-surface-900";
      frame.append(createPreview(item));

      const body = document.createElement("div");
      body.className = "flex flex-col gap-2 p-3";

      const meta = document.createElement("p");
      meta.className = "text-xs font-bold uppercase tracking-wide text-accent-400";
      meta.textContent = `${item.media_type} / ${formatBytes(item.size_bytes)}`;

      const key = document.createElement("p");
      key.className = "break-all font-mono text-xs text-muted";
      key.textContent = item.object_key;

      const button = document.createElement("button");
      button.type = "button";
      button.className = "rounded-lg border border-accent-400/50 px-3 py-2 text-sm font-black text-accent-400 transition hover:bg-accent-500 hover:text-white";
      button.textContent = "Insert";
      button.addEventListener("click", () => insertIntoBody(embedFor(item)));

      body.append(meta, key, button);
      card.append(frame, body);
      mediaGrid.append(card);
    }
  };

  const requestJson = async (url, options = {}) => {
    const response = await fetch(url, {
      credentials: "same-origin",
      ...options,
      headers: {
        ...(options.headers || {}),
      },
    });

    if (!response.ok) {
      let message = "Request failed.";
      try {
        const payload = await response.json();
        message = payload.error || message;
      } catch (_error) {
      }
      throw new Error(message);
    }

    return response.json();
  };

  const loadCategories = async () => {
    const categories = await requestJson("/admin/api/categories");
    categorySelect.replaceChildren();

    for (const category of categories) {
      const option = document.createElement("option");
      option.value = String(category.id);
      option.textContent = category.name;
      categorySelect.append(option);
    }
  };

  const loadPost = async () => {
    if (!isEdit) {
      return;
    }

    if (heading) {
      heading.textContent = "Edit post";
    }

    const post = await requestJson(`/admin/api/posts/${postId}`);
    titleInput.value = post.title;
    categorySelect.value = String(post.category_id);
    statusSelect.value = post.status;
    bodyInput.value = post.body;
  };

  const loadMedia = async () => {
    renderMedia(await requestJson("/admin/api/media"));
  };

  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    setError("");
    setStatus("");

    const button = form.querySelector("button[type='submit']");
    if (button) {
      button.disabled = true;
      button.textContent = "Saving...";
    }

    try {
      const payload = {
        title: titleInput.value,
        category_id: Number(categorySelect.value),
        status: statusSelect.value,
        body: bodyInput.value,
      };
      const saved = await requestJson(isEdit ? `/admin/api/posts/${postId}` : "/admin/api/posts", {
        method: isEdit ? "PUT" : "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });

      setStatus("Saved.");
      if (!isEdit) {
        window.location.assign(`/admin/posts/edit?id=${saved.id}`);
      }
    } catch (error) {
      setError(error instanceof Error ? error.message : "Could not save post.");
    } finally {
      if (button) {
        button.disabled = false;
        button.textContent = "Save";
      }
    }
  });

  mediaUploadForm?.addEventListener("submit", async (event) => {
    event.preventDefault();
    setError("");

    const button = mediaUploadForm.querySelector("button[type='submit']");
    if (button) {
      button.disabled = true;
      button.textContent = "Uploading...";
    }

    try {
      await requestJson("/admin/api/media/upload", {
        method: "POST",
        body: new FormData(mediaUploadForm),
      });
      mediaUploadForm.reset();
      await loadMedia();
    } catch (error) {
      setError(error instanceof Error ? error.message : "Upload failed.");
    } finally {
      if (button) {
        button.disabled = false;
        button.textContent = "Upload";
      }
    }
  });

  Promise.all([loadCategories(), loadMedia()])
    .then(loadPost)
    .catch((error) => {
      setError(error instanceof Error ? error.message : "Could not load editor.");
    });
})();
