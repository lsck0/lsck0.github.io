function renderPost() {
  requestAnimationFrame(function () {
    var el = document.getElementById("post-content");
    if (!el) return;

    // ---- TikZ diagrams ----
    var hasTikz = false;
    el.querySelectorAll("pre.tikz-src").forEach(function (pre) {
      hasTikz = true;
      var wrapper = document.createElement("div");
      wrapper.className = "tikz-diagram";
      wrapper.setAttribute("data-source", pre.textContent);

      var s = document.createElement("script");
      s.type = "text/tikz";
      var libs = pre.getAttribute("data-libs");
      if (libs) s.setAttribute("data-tikz-libraries", libs);
      s.textContent = pre.textContent;

      wrapper.appendChild(s);
      pre.replaceWith(wrapper);
    });

    addCopyBtn(el.querySelectorAll(".tikz-diagram[data-source]"), "copy tex", "Copy TeX source");
    addSearchText(el.querySelectorAll(".tikz-diagram[data-source]"));

    if (hasTikz && window._tikzjaxRender) {
      window._tikzjaxRender();
      pollTikzSize(el);
    }

    // ---- KaTeX math rendering ----
    if (window.renderMathInElement) {
      renderMathInElement(el, {
        delimiters: [
          { left: "$$", right: "$$", display: true },
          { left: "$", right: "$", display: false },
          { left: "\\(", right: "\\)", display: false },
          { left: "\\[", right: "\\]", display: true },
        ],
        throwOnError: false,
      });
      addSearchText(el.querySelectorAll(".math-inline[data-latex], .math-display[data-latex]"));
    }

    // ---- Mermaid diagrams ----
    if (window.mermaid) {
      var theme = document.documentElement.getAttribute("data-theme") || "light";
      var config = getMermaidConfig(theme);
      mermaid.initialize(config);
      var nodes = el.querySelectorAll(".mermaid");
      if (nodes.length > 0) {
        nodes.forEach(function (node) {
          node.setAttribute("data-source", node.textContent);
        });
        mermaid.run({ nodes: nodes }).then(function () {
          addCopyBtn(el.querySelectorAll(".mermaid[data-source]"), "copy mermaid", "Copy Mermaid source");
          setupMermaidZoom(el);
          addSearchText(el.querySelectorAll(".mermaid[data-source]"));
        });
      }
    }

    // ---- Syntax highlighting ----
    if (window.Prism) {
      Prism.highlightAllUnder(el);
    }

    // ---- Code copy buttons ----
    el.querySelectorAll("pre").forEach(function (pre) {
      if (pre.classList.contains("tikz-src") || pre.classList.contains("mermaid") || pre.querySelector(".copy-btn"))
        return;
      var btn = document.createElement("button");
      btn.className = "copy-btn";
      btn.textContent = "copy";
      btn.title = "Copy code to clipboard";
      btn.addEventListener("click", function () {
        var code = pre.querySelector("code");
        var text = code ? code.textContent : pre.textContent;
        navigator.clipboard.writeText(text).then(function () {
          btn.textContent = "copied";
          setTimeout(function () { btn.textContent = "copy"; }, 1500);
        });
      });
      pre.style.position = "relative";
      pre.appendChild(btn);
    });

    // ---- Math copy buttons ----
    el.querySelectorAll(".math-display").forEach(function (div) {
      var latex = div.getAttribute("data-latex");
      if (!latex || div.querySelector(".math-copy-btn")) return;
      var btn = document.createElement("button");
      btn.className = "math-copy-btn";
      btn.textContent = "copy tex";
      btn.title = "Copy TeX source";
      btn.addEventListener("click", function () {
        navigator.clipboard.writeText(latex).then(function () {
          btn.textContent = "copied";
          setTimeout(function () { btn.textContent = "copy tex"; }, 1500);
        });
      });
      div.style.position = "relative";
      div.appendChild(btn);
    });

    // ---- Footnote repositioning (sidenotes on desktop) ----
    repositionFootnotes(el);

    // ---- Hover preview tooltips (unified) ----
    setupTooltips(el);

    // ---- Media embeds ----
    setupMediaEmbeds(el);
  });
}

// ============================================================
// Shared helpers
// ============================================================

function addCopyBtn(nodes, label, title) {
  nodes.forEach(function (node) {
    if (node.querySelector(".math-copy-btn")) return;
    var source = node.getAttribute("data-source");
    if (!source) return;
    var btn = document.createElement("button");
    btn.className = "math-copy-btn";
    btn.textContent = label;
    btn.title = title;
    btn.addEventListener("click", function () {
      navigator.clipboard.writeText(source).then(function () {
        btn.textContent = "copied";
        setTimeout(function () { btn.textContent = label; }, 1500);
      });
    });
    node.style.position = "relative";
    node.appendChild(btn);
  });
}

function addSearchText(nodes) {
  nodes.forEach(function (node) {
    if (node.querySelector(".search-text")) return;
    var source = node.getAttribute("data-source") || node.getAttribute("data-latex");
    if (!source) return;
    var span = document.createElement("span");
    span.className = "search-text";
    span.textContent = source;
    node.appendChild(span);
  });
}

// ============================================================
// TikZ sizing
// ============================================================

function pollTikzSize(el) {
  var tikzPoll = setInterval(function () {
    el.querySelectorAll(".tikz-diagram").forEach(function (wrapper) {
      if (wrapper.dataset.sizeFixed) return;
      var page = wrapper.querySelector(".page");
      if (!page) return;
      var svg = page.querySelector("svg");
      if (!svg) return;
      var w = svg.getAttribute("width");
      var h = svg.getAttribute("height");
      if (!w || !h) return;

      var widthPt = parseFloat(w);
      var heightPt = parseFloat(h);
      if (isNaN(widthPt) || isNaN(heightPt)) return;

      var containerWidth = wrapper.parentElement ? wrapper.parentElement.offsetWidth : 600;
      var scale = Math.min(containerWidth / widthPt, 1.5);

      page.style.setProperty("transform", "scale(" + scale + ")", "important");
      page.style.setProperty("transform-origin", "top center", "important");
      page.style.setProperty("width", w, "important");
      page.style.setProperty("height", h, "important");

      wrapper.style.width = "100%";
      wrapper.style.height = heightPt * scale + "pt";
      wrapper.dataset.sizeFixed = "true";
    });
  }, 500);
  setTimeout(function () { clearInterval(tikzPoll); }, 15000);
}

// ============================================================
// Footnote repositioning
// ============================================================

function repositionFootnotes(el) {
  if (window.innerWidth < 1200) return;
  el.querySelectorAll("sup.footnote-reference").forEach(function (sup) {
    var a = sup.querySelector("a");
    if (!a) return;
    var href = a.getAttribute("href");
    if (!href || !href.startsWith("#")) return;
    var id = href.slice(1);
    var def = el.querySelector("#" + CSS.escape(id));
    if (!def || !def.classList.contains("footnote-definition")) return;
    var para = sup.closest("p, li, td, th, blockquote");
    if (para && para.parentNode) {
      para.parentNode.insertBefore(def, para.nextSibling);
    }
  });
}

// ============================================================
// Unified tooltip system
// ============================================================

var _tooltip = null;

function getTooltip() {
  if (!_tooltip) {
    _tooltip = document.createElement("div");
    _tooltip.className = "xref-tooltip";
    _tooltip.addEventListener("mouseenter", function () {
      _tooltip.classList.add("visible");
    });
    _tooltip.addEventListener("mouseleave", function () {
      _tooltip.classList.remove("visible");
    });
    document.body.appendChild(_tooltip);
  }
  return _tooltip;
}

function positionTooltip(tooltip, anchor) {
  var rect = anchor.getBoundingClientRect();
  var maxWidth = 420;
  var left = Math.min(rect.left, window.innerWidth - maxWidth - 16);
  var top = rect.bottom + 8;
  if (top + 200 > window.innerHeight) {
    top = rect.top - 8;
    tooltip.style.transform = "translateY(-100%)";
  } else {
    tooltip.style.transform = "";
  }
  tooltip.style.left = Math.max(8, left) + "px";
  tooltip.style.top = top + "px";
  tooltip.classList.add("visible");
}

function hideTooltip() {
  if (_tooltip) _tooltip.classList.remove("visible");
}

function setupTooltips(el) {
  // Cross-references and auto-definitions (both have data-preview)
  el.querySelectorAll("[data-preview]").forEach(function (link) {
    link.addEventListener("mouseenter", function () {
      var preview = link.getAttribute("data-preview");
      if (!preview) return;
      var tooltip = getTooltip();
      tooltip.innerHTML = markdownPreview(preview);
      if (window.renderMathInElement) {
        renderMathInElement(tooltip, {
          delimiters: [
            { left: "\\(", right: "\\)", display: false },
            { left: "\\[", right: "\\]", display: true },
          ],
          throwOnError: false,
        });
      }
      positionTooltip(tooltip, link);
    });
    link.addEventListener("mouseleave", hideTooltip);
  });

  // Internal link previews (posts with data-post-title)
  el.querySelectorAll("a[data-post-title]").forEach(function (link) {
    if (link.hasAttribute("data-preview")) return; // already handled above
    link.addEventListener("mouseenter", function () {
      var title = link.getAttribute("data-post-title");
      if (!title) return;
      var desc = link.getAttribute("data-post-desc");
      var tags = link.getAttribute("data-post-tags");
      var series = link.getAttribute("data-post-series");
      var tooltip = getTooltip();
      var html = "<strong>" + title + "</strong>";
      if (desc) html += "<p>" + desc + "</p>";
      if (series) html += '<p style="color:var(--accent);font-size:0.8rem">' + series + "</p>";
      if (tags) html += '<p style="font-size:0.75rem;color:var(--text-muted)">' + tags.split(", ").map(function(t) { return "#" + t; }).join(" ") + "</p>";
      tooltip.innerHTML = html;
      positionTooltip(tooltip, link);
    });
    link.addEventListener("mouseleave", hideTooltip);
  });

  // External link previews
  el.querySelectorAll("a[href^='http']").forEach(function (link) {
    if (link.hasAttribute("data-post-title") || link.hasAttribute("data-preview")) return;
    link.addEventListener("mouseenter", function () {
      var href = link.getAttribute("href");
      if (!href) return;
      try { var url = new URL(href); } catch (e) { return; }
      var tooltip = getTooltip();
      var favicon = "https://www.google.com/s2/favicons?domain=" + url.hostname + "&sz=32";
      var linkText = link.textContent.trim();
      var displayText = linkText !== href ? linkText : "";
      var path = url.pathname + url.search + url.hash;
      if (path.length > 60) path = path.slice(0, 57) + "...";
      var html = '<div class="ext-preview">';
      html += '<div class="ext-preview-header">';
      html += '<img class="ext-favicon" src="' + favicon + '" width="16" height="16" alt="" />';
      html += '<span class="ext-domain">' + url.hostname + '</span></div>';
      if (displayText && displayText !== url.hostname) html += '<div class="ext-title">' + displayText + '</div>';
      if (path !== "/") html += '<div class="ext-path">' + path + '</div>';
      html += '</div>';
      tooltip.innerHTML = html;
      positionTooltip(tooltip, link);
    });
    link.addEventListener("mouseleave", hideTooltip);
  });
}

function markdownPreview(md) {
  return md
    .replace(/```(\w*)\n?([\s\S]*?)```/g, "<pre><code>$2</code></pre>")
    .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
    .replace(/\*(.+?)\*/g, "<em>$1</em>")
    .replace(/`(.+?)`/g, "<code>$1</code>")
    .replace(/\$\$(.+?)\$\$/gs, "\\[$1\\]")
    .replace(/\$(.+?)\$/g, "\\($1\\)")
    .replace(/\n\n/g, "</p><p>")
    .replace(/\n/g, " ")
    .replace(/^/, "<p>")
    .replace(/$/, "</p>");
}

// ============================================================
// Mermaid zoom/pan
// ============================================================

function setupMermaidZoom(el) {
  el.querySelectorAll(".mermaid").forEach(function (container) {
    var svg = container.querySelector("svg");
    if (!svg) return;
    var state = { scale: 1, tx: 0, ty: 0, dragging: false, lastX: 0, lastY: 0 };

    function applyTransform() {
      svg.style.transform = "translate(" + state.tx + "px, " + state.ty + "px) scale(" + state.scale + ")";
      svg.style.transformOrigin = "center center";
    }

    container.addEventListener("wheel", function (e) {
      e.preventDefault();
      var rect = container.getBoundingClientRect();
      var mx = e.clientX - rect.left - rect.width / 2;
      var my = e.clientY - rect.top - rect.height / 2;
      var oldScale = state.scale;
      var delta = e.deltaY > 0 ? 0.9 : 1.1;
      state.scale = Math.max(0.3, Math.min(5, oldScale * delta));
      var ratio = state.scale / oldScale;
      state.tx = mx - ratio * (mx - state.tx);
      state.ty = my - ratio * (my - state.ty);
      applyTransform();
    }, { passive: false });

    container.addEventListener("mousedown", function (e) {
      if (e.button !== 0) return;
      state.dragging = true;
      state.lastX = e.clientX;
      state.lastY = e.clientY;
      container.style.cursor = "grabbing";
    });

    window.addEventListener("mousemove", function (e) {
      if (!state.dragging) return;
      state.tx += e.clientX - state.lastX;
      state.ty += e.clientY - state.lastY;
      state.lastX = e.clientX;
      state.lastY = e.clientY;
      applyTransform();
    });

    window.addEventListener("mouseup", function () {
      if (state.dragging) {
        state.dragging = false;
        container.style.cursor = "grab";
      }
    });

    container.addEventListener("dblclick", function () {
      state.scale = 1;
      state.tx = 0;
      state.ty = 0;
      applyTransform();
    });

    container.style.cursor = "grab";
    container.style.overflow = "hidden";
  });
}

// ============================================================
// Media embeds
// ============================================================

function setupMediaEmbeds(el) {
  el.querySelectorAll(".media-embed").forEach(function (div) {
    var type = div.getAttribute("data-type");
    var src = div.getAttribute("data-src");
    var title = div.getAttribute("data-title") || "";
    if (!type || !src) return;

    if (type === "audio") {
      var audio = document.createElement("audio");
      audio.controls = true;
      audio.preload = "metadata";
      audio.src = src;
      div.appendChild(audio);
    } else if (type === "video") {
      var video = document.createElement("video");
      video.controls = true;
      video.preload = "metadata";
      video.src = src;
      div.appendChild(video);
    } else if (type === "pdf") {
      var obj = document.createElement("object");
      obj.data = src;
      obj.type = "application/pdf";
      obj.width = "100%";
      obj.height = "600";
      var fallback = document.createElement("p");
      fallback.innerHTML = 'PDF cannot be displayed. <a href="' + src + '">Download</a>';
      obj.appendChild(fallback);
      div.appendChild(obj);
    }

    var dl = document.createElement("a");
    dl.href = src;
    dl.download = "";
    dl.className = "media-download";
    dl.textContent = "download " + (title || type);
    div.appendChild(dl);
  });
}
