(() => {
  const root = document.querySelector("[data-media-picker]");
  if (!root || root.dataset.mediaPickerReady === "true") {
    return;
  }
  root.dataset.mediaPickerReady = "true";

  const form = root.querySelector("[data-media-upload-form]");
  const grid = root.querySelector("[data-media-grid]");
  const errorBox = root.querySelector("[data-media-error]");
  const selected = root.querySelector("[data-media-selected]");
  if (!form || !grid || !selected) {
    return;
  }

  const setError = (message) => {
    if (!errorBox) {
      return;
    }

    errorBox.textContent = message || "";
    errorBox.classList.toggle("hidden", !message);
  };

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
    grid.replaceChildren();

    if (!items.length) {
      const empty = document.createElement("div");
      empty.className = "rounded-lg border border-white/10 bg-surface-900 p-4 text-sm font-bold text-muted";
      empty.textContent = "No media yet.";
      grid.append(empty);
      return;
    }

    for (const item of items) {
      const card = document.createElement("article");
      card.className = "overflow-hidden rounded-lg border border-white/10 bg-surface-900";

      const frame = document.createElement("div");
      frame.className = "aspect-video bg-background";
      frame.append(createPreview(item));

      const body = document.createElement("div");
      body.className = "flex flex-col gap-3 p-4";

      const key = document.createElement("p");
      key.className = "break-all font-mono text-xs text-muted";
      key.textContent = item.object_key;

      const meta = document.createElement("p");
      meta.className = "text-xs font-bold uppercase tracking-wide text-accent-400";
      meta.textContent = `${item.media_type} / ${formatBytes(item.size_bytes)}`;

      const button = document.createElement("button");
      button.type = "button";
      button.className = "rounded-lg border border-accent-400/50 px-3 py-2 text-sm font-black text-accent-400 transition hover:bg-accent-500 hover:text-white";
      button.textContent = "Select";
      button.addEventListener("click", () => {
        selected.value = embedFor(item);
        selected.focus();
        selected.select();
      });

      body.append(meta, key, button);
      card.append(frame, body);
      grid.append(card);
    }
  };

  const loadMedia = async () => {
    setError("");
    const response = await fetch("/admin/api/media", { credentials: "same-origin" });
    if (!response.ok) {
      throw new Error("Could not load media.");
    }
    renderMedia(await response.json());
  };

  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    setError("");

    const button = form.querySelector("button[type='submit']");
    if (button) {
      button.disabled = true;
      button.textContent = "Uploading...";
    }

    try {
      const response = await fetch("/admin/api/media/upload", {
        method: "POST",
        body: new FormData(form),
        credentials: "same-origin",
      });

      if (!response.ok) {
        let message = "Upload failed.";
        try {
          const payload = await response.json();
          message = payload.error || message;
        } catch (_error) {
        }
        throw new Error(message);
      }

      form.reset();
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

  loadMedia().catch((error) => {
    setError(error instanceof Error ? error.message : "Could not load media.");
    renderMedia([]);
  });
})();
