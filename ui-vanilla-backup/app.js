(function () {
  "use strict";

  const state = {
    activeTab: "styles",
    styles: {
      q: "",
      theme: "all",
      category: "all",
      searchContent: true,
      limit: 48,
      offset: 0,
      total: 0,
      items: [],
    },
    collections: [],
    templates: {
      lang: "all",
      q: "",
      total: 0,
      items: [],
    },
    licenses: null,
    currentStyleDetail: null,
    currentTemplateDetail: null,
  };

  function escapeHtml(str) {
    return String(str ?? "")
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function showToast(msg) {
    const el = document.getElementById("toast");
    if (!el) return;
    el.textContent = msg;
    el.classList.add("show");
    clearTimeout(el._timer);
    el._timer = setTimeout(() => el.classList.remove("show"), 2200);
  }

  async function copyText(text, label) {
    try {
      await navigator.clipboard.writeText(text);
      showToast(label || "Copied to clipboard!");
    } catch (_) {
      const ta = document.createElement("textarea");
      ta.value = text;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      document.body.removeChild(ta);
      showToast(label || "Copied to clipboard!");
    }
  }

  function renderMarkdown(md) {
    if (!md) return "<p><em>No DESIGN.md file available for this style yet.</em></p>";
    const lines = String(md).split(/\r?\n/);
    let html = "";
    let inCode = false;
    let codeBuf = [];
    let inList = false;

    const formatInline = (text) => {
      let escaped = escapeHtml(text);
      escaped = escaped.replace(/`([^`]+)`/g, (_, code) => {
        const hexMatch = code.match(/^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})$/);
        const swatch = hexMatch
          ? `<span class="color-swatch-inline" style="background:${code}"></span>`
          : "";
        return `<code>${swatch}${code}</code>`;
      });
      escaped = escaped.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
      escaped = escaped.replace(/\*([^*]+)\*/g, "<em>$1</em>");
      return escaped;
    };

    for (const line of lines) {
      if (line.trim().startsWith("```")) {
        if (inList) {
          html += "</ul>";
          inList = false;
        }
        if (!inCode) {
          inCode = true;
          codeBuf = [];
        } else {
          html += `<pre><code>${escapeHtml(codeBuf.join("\n"))}</code></pre>`;
          inCode = false;
        }
        continue;
      }
      if (inCode) {
        codeBuf.push(line);
        continue;
      }

      const trimmed = line.trim();
      if (!trimmed) {
        if (inList) {
          html += "</ul>";
          inList = false;
        }
        continue;
      }

      if (trimmed.startsWith("#### ")) {
        if (inList) { html += "</ul>"; inList = false; }
        html += `<h4>${formatInline(trimmed.slice(5))}</h4>`;
      } else if (trimmed.startsWith("### ")) {
        if (inList) { html += "</ul>"; inList = false; }
        html += `<h3>${formatInline(trimmed.slice(4))}</h3>`;
      } else if (trimmed.startsWith("## ")) {
        if (inList) { html += "</ul>"; inList = false; }
        html += `<h2>${formatInline(trimmed.slice(3))}</h2>`;
      } else if (trimmed.startsWith("# ")) {
        if (inList) { html += "</ul>"; inList = false; }
        html += `<h1>${formatInline(trimmed.slice(2))}</h1>`;
      } else if (trimmed.startsWith("- ") || trimmed.startsWith("* ")) {
        if (!inList) {
          html += "<ul>";
          inList = true;
        }
        html += `<li>${formatInline(trimmed.slice(2))}</li>`;
      } else {
        if (inList) {
          html += "</ul>";
          inList = false;
        }
        html += `<p>${formatInline(trimmed)}</p>`;
      }
    }
    if (inList) html += "</ul>";
    if (inCode) html += `<pre><code>${escapeHtml(codeBuf.join("\n"))}</code></pre>`;
    return `<div class="md-rendered">${html}</div>`;
  }

  function domainFromUrl(url) {
    if (!url) return "";
    try {
      return new URL(url).hostname.replace(/^www\./, "");
    } catch (_) {
      return url;
    }
  }

  async function loadCollections() {
    const res = await fetch("/api/collections");
    const data = await res.json();
    state.collections = data.items || [];
    document.getElementById("badge-collections").textContent = state.collections.length;
    renderCategoryChips();
    renderCollectionsGrid();
  }

  function renderCategoryChips() {
    const row = document.getElementById("category-chips-row");
    if (!row) return;
    const categoryCols = state.collections.filter((c) => c.kind === "category");
    let html = `<button class="chip ${state.styles.category === "all" ? "active" : ""}" data-category="all">All Categories</button>`;
    for (const col of categoryCols) {
      const active = state.styles.category === col.id ? "active" : "";
      html += `<button class="chip ${active}" data-category="${escapeHtml(col.id)}">${escapeHtml(col.name)} (${col.count})</button>`;
    }
    row.innerHTML = html;
    row.querySelectorAll(".chip").forEach((btn) => {
      btn.addEventListener("click", () => {
        state.styles.category = btn.getAttribute("data-category") || "all";
        state.styles.offset = 0;
        renderCategoryChips();
        loadStyles();
      });
    });
  }

  async function loadStyles() {
    const { q, theme, category, searchContent, limit, offset } = state.styles;
    const grid = document.getElementById("styles-grid");
    const summary = document.getElementById("styles-summary-text");

    let items = [];
    let total = 0;

    if (q.trim() && searchContent) {
      const params = new URLSearchParams({
        q: q.trim(),
        include_content: "true",
        limit: String(limit),
      });
      if (theme !== "all") params.set("theme", theme);
      if (category !== "all") params.set("category", category);
      const res = await fetch(`/api/search?${params.toString()}`);
      const data = await res.json();
      items = data.styles || [];
      total = data.total_styles || items.length;
    } else {
      const params = new URLSearchParams({
        limit: String(limit),
        offset: String(offset),
      });
      if (q.trim()) params.set("q", q.trim());
      if (theme !== "all") params.set("theme", theme);
      if (category !== "all") params.set("category", category);
      const res = await fetch(`/api/styles?${params.toString()}`);
      const data = await res.json();
      items = data.items || [];
      total = data.total || 0;
      if (!q.trim() && theme === "all" && category === "all") {
        document.getElementById("badge-styles").textContent = total;
      }
    }

    state.styles.items = items;
    state.styles.total = total;
    summary.textContent = `Showing ${items.length} of ${total} design styles`;

    if (!items.length) {
      grid.innerHTML = `<div class="license-box" style="grid-column:1/-1;text-align:center;color:var(--text-secondary);">No styles matched your current filter criteria.</div>`;
      document.getElementById("styles-pagination").innerHTML = "";
      return;
    }

    grid.innerHTML = items
      .map((s) => {
        const domain = domainFromUrl(s.source_url);
        const cats = (s.categories || [])
          .slice(0, 3)
          .map((c) => `<span class="mini-tag">${escapeHtml(c)}</span>`)
          .join("");
        const snippet = s.match_snippet
          ? `<div class="card-snippet">${escapeHtml(s.match_snippet)}</div>`
          : "";
        return `
          <article class="style-card" data-style-id="${escapeHtml(s.id)}">
            <div>
              <div class="card-swatch-bar ${escapeHtml(s.theme)}"></div>
              <div class="card-header">
                <h3 class="card-title">${escapeHtml(s.title)}</h3>
                <span class="theme-badge ${escapeHtml(s.theme)}">${escapeHtml(s.theme)}</span>
              </div>
              <p class="card-tagline" style="margin-top:0.5rem;">${escapeHtml(s.tagline || "Design system specification & UI tokens.")}</p>
              ${snippet ? `<div style="margin-top:0.55rem;">${snippet}</div>` : ""}
            </div>
            <div class="card-footer">
              <div class="card-tags">${cats}</div>
              ${
                domain
                  ? `<a href="${escapeHtml(s.source_url)}" target="_blank" rel="noopener" onclick="event.stopPropagation()" title="Open product site">${escapeHtml(domain)} &nearr;</a>`
                  : `<span>DESIGN.md</span>`
              }
            </div>
          </article>
        `;
      })
      .join("");

    grid.querySelectorAll(".style-card").forEach((card) => {
      card.addEventListener("click", () => {
        const sid = card.getAttribute("data-style-id");
        if (sid) openStyleModal(sid);
      });
    });

    renderStylesPagination();
  }

  function renderStylesPagination() {
    const bar = document.getElementById("styles-pagination");
    if (!bar) return;
    if (state.styles.q.trim() && state.styles.searchContent) {
      bar.innerHTML = "";
      return;
    }
    const { total, limit, offset } = state.styles;
    if (total <= limit) {
      bar.innerHTML = "";
      return;
    }
    const currentPage = Math.floor(offset / limit) + 1;
    const totalPages = Math.ceil(total / limit);
    bar.innerHTML = `
      <button class="btn" id="page-prev" ${offset <= 0 ? "disabled" : ""}>&larr; Previous</button>
      <span style="font-size:0.85rem;color:var(--text-secondary);">Page ${currentPage} of ${totalPages} (${total} styles)</span>
      <button class="btn" id="page-next" ${offset + limit >= total ? "disabled" : ""}>Next &rarr;</button>
    `;
    const prev = document.getElementById("page-prev");
    const next = document.getElementById("page-next");
    if (prev) {
      prev.addEventListener("click", () => {
        if (state.styles.offset >= state.styles.limit) {
          state.styles.offset -= state.styles.limit;
          loadStyles();
          window.scrollTo({ top: 0, behavior: "smooth" });
        }
      });
    }
    if (next) {
      next.addEventListener("click", () => {
        if (state.styles.offset + state.styles.limit < state.styles.total) {
          state.styles.offset += state.styles.limit;
          loadStyles();
          window.scrollTo({ top: 0, behavior: "smooth" });
        }
      });
    }
  }

  function renderCollectionsGrid() {
    const grid = document.getElementById("collections-grid");
    if (!grid) return;
    grid.innerHTML = state.collections
      .map((col) => {
        const samples = (col.sample_styles || [])
          .slice(0, 4)
          .map((s) => `<span class="mini-tag">${escapeHtml(s.title)}</span>`)
          .join("");
        return `
          <article class="collection-card" data-col-id="${escapeHtml(col.id)}" data-col-kind="${escapeHtml(col.kind)}">
            <div>
              <div style="height:5px;border-radius:999px;background:${escapeHtml(col.accent || "#6366f1")};margin-bottom:0.75rem;"></div>
              <div class="card-header">
                <h3 class="card-title">${escapeHtml(col.name)}</h3>
                <span class="nav-badge" style="background:var(--bg-elevated);color:var(--text-primary);font-weight:700;">${col.count} styles</span>
              </div>
              <p class="card-tagline" style="margin-top:0.5rem;">${escapeHtml(col.description)}</p>
            </div>
            <div>
              <div class="collection-samples" style="margin-bottom:0.75rem;">${samples}</div>
              <div class="card-footer">
                <span style="text-transform:uppercase;font-size:0.68rem;letter-spacing:0.05em;">${escapeHtml(col.kind)} collection</span>
                <span style="color:#818cf8;font-weight:600;">Explore &rarr;</span>
              </div>
            </div>
          </article>
        `;
      })
      .join("");

    grid.querySelectorAll(".collection-card").forEach((card) => {
      card.addEventListener("click", () => {
        const cid = card.getAttribute("data-col-id") || "all";
        if (cid === "theme-dark") {
          state.styles.theme = "dark";
          state.styles.category = "all";
        } else if (cid === "theme-light") {
          state.styles.theme = "light";
          state.styles.category = "all";
        } else {
          state.styles.category = cid;
          state.styles.theme = "all";
        }
        state.styles.offset = 0;
        syncThemeButtons();
        renderCategoryChips();
        window.location.hash = "#styles";
        loadStyles();
      });
    });
  }

  async function loadTemplates() {
    const { lang, q } = state.templates;
    const params = new URLSearchParams({ limit: "150", offset: "0" });
    if (lang !== "all") params.set("lang", lang);
    if (q.trim()) params.set("q", q.trim());

    const res = await fetch(`/api/templates?${params.toString()}`);
    const data = await res.json();
    const items = data.items || [];
    state.templates.items = items;
    state.templates.total = data.total || items.length;

    if (lang === "all" && !q.trim()) {
      document.getElementById("badge-templates").textContent = state.templates.total;
    }

    document.getElementById("templates-summary-text").textContent =
      `Showing ${items.length} of ${state.templates.total} full-HTML templates (prompt-driven-ui)`;

    const grid = document.getElementById("templates-grid");
    if (!items.length) {
      grid.innerHTML = `<div class="license-box" style="grid-column:1/-1;text-align:center;color:var(--text-secondary);">No HTML templates matched your search.</div>`;
      return;
    }

    grid.innerHTML = items
      .map((tpl) => {
        return `
          <article class="template-card">
            <div class="template-thumb-wrap">
              <img class="template-thumb" src="${escapeHtml(tpl.thumb_url)}" alt="${escapeHtml(tpl.title)}" loading="lazy" />
            </div>
            <div class="template-body">
              <div class="card-header">
                <h3 class="card-title">${escapeHtml(tpl.title)}</h3>
                <span class="theme-badge ${tpl.lang === "en" ? "dark" : "light"}">${escapeHtml(tpl.lang.toUpperCase())} #${String(tpl.number).padStart(2, "0")}</span>
              </div>
              <p class="card-tagline">${escapeHtml(tpl.prompt_preview || tpl.slug)}</p>
              <div class="template-actions">
                <a class="btn btn-primary" href="${escapeHtml(tpl.html_url)}" target="_blank" rel="noopener">Open HTML &nearr;</a>
                <button class="btn btn-inspect-tpl" data-tpl-id="${escapeHtml(tpl.id)}">Preview &amp; Prompt</button>
              </div>
            </div>
          </article>
        `;
      })
      .join("");

    grid.querySelectorAll(".btn-inspect-tpl").forEach((btn) => {
      btn.addEventListener("click", () => {
        const tid = btn.getAttribute("data-tpl-id");
        if (tid) openTemplateModal(tid);
      });
    });
  }

  async function loadLicenses() {
    const container = document.getElementById("licenses-content");
    if (!state.licenses) {
      const res = await fetch("/api/licenses");
      state.licenses = await res.json();
    }
    const data = state.licenses;
    const corporaHtml = (data.corpora || [])
      .map(
        (c) => `
        <div class="license-box">
          <div class="card-header" style="margin-bottom:0.5rem;">
            <h3 style="font-size:1.1rem;font-weight:700;">${escapeHtml(c.name)}</h3>
            <a href="${escapeHtml(c.upstream_url)}" target="_blank" rel="noopener">${escapeHtml(c.upstream_url)} &nearr;</a>
          </div>
          <p style="color:var(--text-secondary);font-size:0.9rem;margin-bottom:0.5rem;">${escapeHtml(c.license_note)}</p>
          <div style="font-size:0.8rem;color:var(--text-muted);">
            ${c.license ? `<strong>License:</strong> ${escapeHtml(c.license)} &bull; ` : ""}
            ${c.total_styles !== undefined ? `<strong>Indexed Styles:</strong> ${c.total_styles}` : ""}
            ${c.total_templates !== undefined ? `<strong>Indexed Templates:</strong> ${c.total_templates}` : ""}
          </div>
        </div>
      `
      )
      .join("");

    const sources = data.attributed_sources || [];
    container.innerHTML = `
      ${corporaHtml}
      <div class="license-box">
        <div class="toolbar-row" style="margin-bottom:1rem;">
          <div>
            <h3 style="font-size:1.05rem;font-weight:700;">Attributed Product Websites (${sources.length})</h3>
            <p style="font-size:0.82rem;color:var(--text-secondary);">Each Refero style is distilled from the live product website listed below.</p>
          </div>
          <input type="search" id="license-filter-input" class="search-input" style="max-width:320px;" placeholder="Filter source websites..." />
        </div>
        <div style="max-height:540px;overflow-y:auto;border:1px solid var(--border-subtle);border-radius:var(--radius-md);">
          <table class="source-table">
            <thead>
              <tr>
                <th>Style Title</th>
                <th>Theme</th>
                <th>Product Website (source_url)</th>
                <th>Refero Entry</th>
              </tr>
            </thead>
            <tbody id="license-sources-tbody"></tbody>
          </table>
        </div>
      </div>
    `;

    const renderRows = (filterText) => {
      const q = (filterText || "").toLowerCase().trim();
      const filtered = q
        ? sources.filter((s) => `${s.title} ${s.source_url} ${s.id}`.toLowerCase().includes(q))
        : sources.slice(0, 300);
      const tbody = document.getElementById("license-sources-tbody");
      if (!tbody) return;
      tbody.innerHTML = filtered
        .map(
          (s) => `
          <tr>
            <td><strong>${escapeHtml(s.title)}</strong></td>
            <td><span class="theme-badge ${escapeHtml(s.theme)}">${escapeHtml(s.theme)}</span></td>
            <td>${s.source_url ? `<a href="${escapeHtml(s.source_url)}" target="_blank" rel="noopener">${escapeHtml(s.source_url)} &nearr;</a>` : "&mdash;"}</td>
            <td><a href="${escapeHtml(s.refero_url)}" target="_blank" rel="noopener">Refero &nearr;</a></td>
          </tr>
        `
        )
        .join("");
    };

    renderRows("");
    const filterInput = document.getElementById("license-filter-input");
    if (filterInput) {
      filterInput.addEventListener("input", (e) => renderRows(e.target.value));
    }
  }

  async function openStyleModal(styleId) {
    const modal = document.getElementById("detail-modal");
    const titleEl = document.getElementById("modal-title");
    const subEl = document.getElementById("modal-subtitle");
    const toolbarEl = document.getElementById("modal-toolbar");
    const bodyEl = document.getElementById("modal-body");

    modal.classList.add("open");
    titleEl.textContent = "Loading DESIGN.md...";
    subEl.textContent = styleId;
    toolbarEl.innerHTML = "";
    bodyEl.innerHTML = "<p>Fetching design specification...</p>";

    const res = await fetch(`/api/styles/${encodeURIComponent(styleId)}`);
    if (!res.ok) {
      bodyEl.innerHTML = `<p>Could not load style ${escapeHtml(styleId)}.</p>`;
      return;
    }
    const item = await res.json();
    state.currentStyleDetail = item;

    titleEl.textContent = item.title;
    subEl.textContent = item.tagline || item.id;

    const renderView = (mode) => {
      toolbarEl.querySelectorAll(".seg-btn").forEach((b) => {
        b.classList.toggle("active", b.getAttribute("data-view") === mode);
      });
      if (mode === "rendered") {
        bodyEl.innerHTML = renderMarkdown(item.design_md);
      } else if (mode === "raw") {
        bodyEl.innerHTML = `<pre class="raw-pre">${escapeHtml(item.design_md || "")}</pre>`;
      } else if (mode === "meta") {
        const metaPayload = {
          id: item.id,
          title: item.title,
          tagline: item.tagline,
          theme: item.theme,
          url: item.url,
          source_url: item.source_url,
          categories: item.categories,
          meta: item.meta,
        };
        bodyEl.innerHTML = `<pre class="raw-pre">${escapeHtml(JSON.stringify(metaPayload, null, 2))}</pre>`;
      }
    };

    toolbarEl.innerHTML = `
      <div class="segmented-group">
        <button class="seg-btn active" data-view="rendered">Rendered DESIGN.md</button>
        <button class="seg-btn" data-view="raw">Raw Markdown</button>
        <button class="seg-btn" data-view="meta">Metadata JSON</button>
      </div>
      <div style="display:flex;align-items:center;gap:0.5rem;flex-wrap:wrap;">
        <button class="btn btn-primary" id="btn-copy-design">Copy DESIGN.md</button>
        ${
          item.source_url
            ? `<a class="btn" href="${escapeHtml(item.source_url)}" target="_blank" rel="noopener">Product Site &nearr;</a>`
            : ""
        }
        ${
          item.url
            ? `<a class="btn" href="${escapeHtml(item.url)}" target="_blank" rel="noopener">Refero Page &nearr;</a>`
            : ""
        }
      </div>
    `;

    toolbarEl.querySelectorAll(".seg-btn").forEach((btn) => {
      btn.addEventListener("click", () => renderView(btn.getAttribute("data-view")));
    });
    const copyBtn = document.getElementById("btn-copy-design");
    if (copyBtn) {
      copyBtn.addEventListener("click", () =>
        copyText(item.design_md || "", `Copied DESIGN.md for ${item.title}`)
      );
    }

    renderView("rendered");
  }

  async function openTemplateModal(templateId) {
    const modal = document.getElementById("detail-modal");
    const titleEl = document.getElementById("modal-title");
    const subEl = document.getElementById("modal-subtitle");
    const toolbarEl = document.getElementById("modal-toolbar");
    const bodyEl = document.getElementById("modal-body");

    modal.classList.add("open");
    titleEl.textContent = "Loading Template...";
    subEl.textContent = templateId;
    toolbarEl.innerHTML = "";
    bodyEl.innerHTML = "<p>Fetching template details...</p>";

    const res = await fetch(`/api/templates/${encodeURIComponent(templateId)}`);
    if (!res.ok) {
      bodyEl.innerHTML = `<p>Could not load template ${escapeHtml(templateId)}.</p>`;
      return;
    }
    const tpl = await res.json();
    state.currentTemplateDetail = tpl;

    titleEl.textContent = `${tpl.title} (${tpl.lang.toUpperCase()} #${String(tpl.number).padStart(2, "0")})`;
    subEl.textContent = tpl.slug;

    const renderView = (mode) => {
      toolbarEl.querySelectorAll(".seg-btn").forEach((b) => {
        b.classList.toggle("active", b.getAttribute("data-view") === mode);
      });
      if (mode === "preview") {
        bodyEl.innerHTML = `
          <iframe
            src="${escapeHtml(tpl.html_url)}"
            style="width:100%;height:68vh;border:1px solid var(--border-subtle);border-radius:var(--radius-md);background:#fff;"
            title="${escapeHtml(tpl.title)}"
          ></iframe>
        `;
      } else if (mode === "prompt") {
        bodyEl.innerHTML = tpl.prompt
          ? `<pre class="raw-pre">${escapeHtml(tpl.prompt)}</pre>`
          : `<p style="color:var(--text-secondary);">No separate prompt file found for this template.</p>`;
      }
    };

    toolbarEl.innerHTML = `
      <div class="segmented-group">
        <button class="seg-btn active" data-view="preview">Live HTML Preview</button>
        <button class="seg-btn" data-view="prompt">Generation Prompt</button>
      </div>
      <div style="display:flex;align-items:center;gap:0.5rem;">
        ${
          tpl.prompt
            ? `<button class="btn" id="btn-copy-prompt">Copy Prompt</button>`
            : ""
        }
        <a class="btn btn-primary" href="${escapeHtml(tpl.html_url)}" target="_blank" rel="noopener">Open Full HTML &nearr;</a>
      </div>
    `;

    toolbarEl.querySelectorAll(".seg-btn").forEach((btn) => {
      btn.addEventListener("click", () => renderView(btn.getAttribute("data-view")));
    });
    const copyBtn = document.getElementById("btn-copy-prompt");
    if (copyBtn) {
      copyBtn.addEventListener("click", () => copyText(tpl.prompt || "", "Copied generation prompt!"));
    }

    renderView("preview");
  }

  function closeModal() {
    document.getElementById("detail-modal").classList.remove("open");
  }

  function syncThemeButtons() {
    document.querySelectorAll("#theme-filter-group .seg-btn").forEach((btn) => {
      btn.classList.toggle("active", btn.getAttribute("data-theme") === state.styles.theme);
    });
  }

  function activateTabFromHash() {
    const rawHash = (window.location.hash || "#styles").replace(/^#/, "").trim();
    const validTabs = ["styles", "collections", "templates", "licenses"];
    const tab = validTabs.includes(rawHash) ? rawHash : "styles";
    state.activeTab = tab;

    document.querySelectorAll(".nav-tab").forEach((el) => {
      el.classList.toggle("active", el.getAttribute("data-tab") === tab);
    });
    document.querySelectorAll(".tab-panel").forEach((panel) => {
      panel.classList.toggle("active", panel.id === `panel-${tab}`);
    });

    if (tab === "styles") loadStyles();
    if (tab === "collections") renderCollectionsGrid();
    if (tab === "templates") loadTemplates();
    if (tab === "licenses") loadLicenses();
  }

  function bindEvents() {
    window.addEventListener("hashchange", activateTabFromHash);

    let searchTimer = null;
    const styleSearch = document.getElementById("styles-search-input");
    if (styleSearch) {
      styleSearch.addEventListener("input", (e) => {
        clearTimeout(searchTimer);
        searchTimer = setTimeout(() => {
          state.styles.q = e.target.value || "";
          state.styles.offset = 0;
          loadStyles();
        }, 240);
      });
    }

    const contentCheck = document.getElementById("styles-search-content");
    if (contentCheck) {
      contentCheck.addEventListener("change", (e) => {
        state.styles.searchContent = Boolean(e.target.checked);
        if (state.styles.q.trim()) loadStyles();
      });
    }

    document.querySelectorAll("#theme-filter-group .seg-btn").forEach((btn) => {
      btn.addEventListener("click", () => {
        state.styles.theme = btn.getAttribute("data-theme") || "all";
        state.styles.offset = 0;
        syncThemeButtons();
        loadStyles();
      });
    });

    let tplTimer = null;
    const tplSearch = document.getElementById("templates-search-input");
    if (tplSearch) {
      tplSearch.addEventListener("input", (e) => {
        clearTimeout(tplTimer);
        tplTimer = setTimeout(() => {
          state.templates.q = e.target.value || "";
          loadTemplates();
        }, 200);
      });
    }

    document.querySelectorAll("#lang-filter-group .seg-btn").forEach((btn) => {
      btn.addEventListener("click", () => {
        state.templates.lang = btn.getAttribute("data-lang") || "all";
        document.querySelectorAll("#lang-filter-group .seg-btn").forEach((b) => {
          b.classList.toggle("active", b.getAttribute("data-lang") === state.templates.lang);
        });
        loadTemplates();
      });
    });

    const reindexBtn = document.getElementById("btn-reindex");
    if (reindexBtn) {
      reindexBtn.addEventListener("click", async () => {
        reindexBtn.disabled = true;
        reindexBtn.textContent = "Re-indexing...";
        try {
          const res = await fetch("/api/admin/reindex", { method: "POST" });
          const stat = await res.json();
          showToast(`Re-indexed ${stat.styles_count} styles & ${stat.templates_count} templates in ${stat.last_index_duration_ms}ms`);
          await Promise.all([loadCollections(), loadStyles(), loadTemplates()]);
        } catch (_) {
          showToast("Re-index failed");
        } finally {
          reindexBtn.disabled = false;
          reindexBtn.innerHTML = "&#x21bb; Re-Index";
        }
      });
    }

    document.getElementById("modal-close-btn").addEventListener("click", closeModal);
    document.getElementById("detail-modal").addEventListener("click", (e) => {
      if (e.target.id === "detail-modal") closeModal();
    });
    window.addEventListener("keydown", (e) => {
      if (e.key === "Escape") closeModal();
    });
  }

  async function init() {
    bindEvents();
    await Promise.all([loadCollections(), loadTemplates()]);
    activateTabFromHash();
  }

  document.addEventListener("DOMContentLoaded", init);
})();
