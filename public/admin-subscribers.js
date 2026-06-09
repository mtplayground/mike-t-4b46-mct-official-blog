(() => {
  const root = document.querySelector("[data-admin-subscribers]");
  if (!root || root.dataset.adminSubscribersReady === "true") {
    return;
  }
  root.dataset.adminSubscribersReady = "true";

  const table = root.querySelector("[data-admin-subscribers-table]");
  const errorBox = root.querySelector("[data-admin-subscribers-error]");
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

  const formatDateTime = (value) => {
    if (!value) {
      return "";
    }

    const date = new Date(value);
    if (Number.isNaN(date.getTime())) {
      return value;
    }

    return date.toLocaleString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    });
  };

  const createCell = (className) => {
    const cell = document.createElement("td");
    cell.className = className;
    return cell;
  };

  const renderEmpty = () => {
    const row = document.createElement("tr");
    const cell = createCell("px-4 py-6 text-sm font-bold text-muted");
    cell.colSpan = 3;
    cell.textContent = "No subscribers yet.";
    row.append(cell);
    table.replaceChildren(row);
  };

  const renderSubscribers = (subscribers) => {
    if (!subscribers.length) {
      renderEmpty();
      return;
    }

    const rows = subscribers.map((subscriber) => {
      const row = document.createElement("tr");

      const email = createCell("px-4 py-4 font-bold text-foreground");
      email.textContent = subscriber.email;

      const created = createCell("px-4 py-4 text-muted");
      created.textContent = formatDateTime(subscriber.created_at);

      const id = createCell("px-4 py-4 text-right font-mono text-xs text-muted");
      id.textContent = String(subscriber.id);

      row.append(email, created, id);
      return row;
    });

    table.replaceChildren(...rows);
  };

  const loadSubscribers = async () => {
    setError("");
    const response = await fetch("/admin/api/subscribers", {
      credentials: "same-origin",
    });

    if (!response.ok) {
      throw new Error("Could not load subscribers.");
    }

    renderSubscribers(await response.json());
  };

  loadSubscribers().catch((error) => {
    setError(error instanceof Error ? error.message : "Could not load subscribers.");
    renderEmpty();
  });
})();
