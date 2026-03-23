function renderPost() {
  requestAnimationFrame(function () {
    var el = document.getElementById("post-content");
    if (!el) {
      // Also handle listing pages (projects, publications)
      var listingEl = document.getElementById("listing-content");
      if (listingEl) {
        if (window.renderMathInElement) {
          renderMathInElement(listingEl, {
            delimiters: [
              { left: "$$", right: "$$", display: true },
              { left: "$", right: "$", display: false },
              { left: "\\(", right: "\\)", display: false },
              { left: "\\[", right: "\\]", display: true },
            ],
            throwOnError: false,
          });
        }
        setupTooltips(listingEl);
        setupMediaEmbeds(listingEl);
      }
      return;
    }

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
      if (libs) {
        s.setAttribute("data-tikz-libraries", libs);
        wrapper.setAttribute("data-tikz-libs", libs);
      }
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
        }).catch(function () {});
      }
    }

    // ---- Syntax highlighting ----
    if (window.Prism) {
      Prism.highlightAllUnder(el);
    }

    // ---- Copy buttons (code, math, tikz, mermaid) ----
    el.querySelectorAll("pre").forEach(function (pre) {
      if (pre.classList.contains("tikz-src") || pre.classList.contains("mermaid") || pre.querySelector(".copy-btn"))
        return;
      var code = pre.querySelector("code");
      var text = code ? code.textContent : pre.textContent;
      addCopyBtn([pre], "copy", "Copy code to clipboard", text);
    });
    el.querySelectorAll(".math-display").forEach(function (div) {
      addCopyBtn([div], "copy tex", "Copy TeX source", div.getAttribute("data-latex"));
    });

    // ---- Pin buttons on labeled blocks ----
    setupBlockPinButtons(el);

    // ---- Footnote repositioning (sidenotes on desktop) ----
    repositionFootnotes(el);

    // ---- Hover preview tooltips (unified) ----
    setupTooltips(el);

    // ---- Tooltips on pinned panel ----
    var pinnedPanel = document.querySelector(".pinned-panel");
    if (pinnedPanel) setupTooltips(pinnedPanel);

    // ---- Media embeds ----
    setupMediaEmbeds(el);
  });
}

// ============================================================
// Shared helpers
// ============================================================

function addCopyBtn(nodes, label, title, textOverride) {
  nodes.forEach(function (node) {
    if (node.querySelector(".copy-btn")) return;
    var source = textOverride || node.getAttribute("data-source");
    if (!source) return;
    var btn = document.createElement("button");
    btn.className = "copy-btn";
    btn.textContent = label;
    btn.title = title;
    btn.addEventListener("click", function () {
      navigator.clipboard.writeText(source).then(function () {
        btn.textContent = "copied";
        setTimeout(function () { btn.textContent = label; }, 1500);
      }).catch(function () {
        btn.textContent = "failed";
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
      var isTikzCd = wrapper.getAttribute("data-tikz-libs") === "cd";
      var maxScale = isTikzCd ? 1.8 : 1.5;
      var baseScale = isTikzCd ? 1.2 : 1.0;
      var scale = Math.min(containerWidth / widthPt * baseScale, maxScale);

      page.style.setProperty("transform", "scale(" + scale + ")");
      page.style.setProperty("transform-origin", "top center");
      page.style.setProperty("width", w);
      page.style.setProperty("height", h);

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
  if (window.innerWidth < 1200) {
    // On mobile: move all footnotes to before the references section
    var footnotes = Array.from(el.querySelectorAll(".footnote-definition"));
    if (footnotes.length === 0) return;

    var content = el;
    var referencesSection = content.querySelector(".post-references");
    var footnotesContainer = document.createElement("div");
    footnotesContainer.className = "footnotes-section";

    var header = document.createElement("h2");
    header.textContent = "Footnotes";
    footnotesContainer.appendChild(header);

    footnotes.forEach(function (footnote) {
      footnote.parentNode.removeChild(footnote);
      footnotesContainer.appendChild(footnote);
    });

    if (referencesSection) {
      content.insertBefore(footnotesContainer, referencesSection);
    } else {
      content.appendChild(footnotesContainer);
    }
  } else {
    // On desktop: move footnotes to sidebar as before
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
}

// ============================================================
// Unified tooltip system
// ============================================================

// ============================================================
// Tooltip stack (supports nested hovers)
// ============================================================

var _tooltipStack = [];
var _tooltipShowTimer = null;
var _tooltipCooldown = 0;
var _tooltipCooldownDuration = 150; // ms before re-showing after hide
var _ogMetadata = null;

// Load pre-fetched OG metadata for external links
fetch("/og_external.json")
  .then(function (r) { return r.ok ? r.json() : {}; })
  .then(function (data) { _ogMetadata = data; })
  .catch(function () { _ogMetadata = {}; });

function escapeHtml(text) {
  var div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

function createTooltip(depth) {
  var tooltip = document.createElement("div");
  tooltip.className = "xref-tooltip";
  tooltip.style.zIndex = 1001 + depth;
  tooltip._depth = depth;
  tooltip._hideTimer = null;

  tooltip.addEventListener("mouseenter", function () {
    clearTimeout(tooltip._hideTimer);
    // Also cancel hide on parent tooltips
    for (var i = 0; i < depth; i++) {
      if (_tooltipStack[i]) clearTimeout(_tooltipStack[i]._hideTimer);
    }
    tooltip.classList.add("visible");
  });
  tooltip.addEventListener("mouseleave", function () {
    scheduleHideTooltipAt(depth);
    // Also schedule hide on parent tooltips (they'll check child visibility)
    for (var i = depth - 1; i >= 0; i--) {
      scheduleHideTooltipAt(i);
    }
  });

  document.body.appendChild(tooltip);
  return tooltip;
}

function getTooltipAt(depth) {
  while (_tooltipStack.length <= depth) {
    _tooltipStack.push(createTooltip(_tooltipStack.length));
  }
  return _tooltipStack[depth];
}

function positionTooltip(tooltip, anchor) {
  var depth = tooltip._depth || 0;
  clearTimeout(tooltip._hideTimer);
  // Cancel hide timers on all parent tooltips (keep them visible)
  for (var i = 0; i < depth; i++) {
    if (_tooltipStack[i]) clearTimeout(_tooltipStack[i]._hideTimer);
  }
  tooltip.style.pointerEvents = "auto";
  var rect = anchor.getBoundingClientRect();
  var maxWidth = 600;
  var offset = depth * 12; // Stack offset for nested tooltips
  var left = Math.min(rect.left + offset, window.innerWidth - maxWidth - 16);
  var top = rect.bottom + 8 + offset;
  if (top + 250 > window.innerHeight) {
    top = rect.top - 8 - offset;
    tooltip.style.transform = "translateY(-100%)";
  } else {
    tooltip.style.transform = "";
  }
  tooltip.style.left = Math.max(8, left) + "px";
  tooltip.style.top = top + "px";
  tooltip.classList.add("visible");
}

function isChildTooltipVisible(depth) {
  for (var i = depth + 1; i < _tooltipStack.length; i++) {
    if (_tooltipStack[i].classList.contains("visible")) return true;
  }
  return false;
}

function scheduleHideTooltipAt(depth) {
  var tooltip = _tooltipStack[depth];
  if (!tooltip) return;
  clearTimeout(tooltip._hideTimer);
  tooltip._hideTimer = setTimeout(function () {
    // Don't hide if a deeper tooltip is still visible (user moved into it)
    if (isChildTooltipVisible(depth)) return;
    tooltip.classList.remove("visible");
    tooltip.style.pointerEvents = "none";
    // Also hide deeper tooltips
    for (var i = depth + 1; i < _tooltipStack.length; i++) {
      _tooltipStack[i].classList.remove("visible");
      _tooltipStack[i].style.pointerEvents = "none";
    }
    if (depth === 0) {
      _tooltipCooldown = Date.now();
    }
  }, 200);
}

function hideTooltip() {
  clearTimeout(_tooltipShowTimer);
  for (var i = 0; i < _tooltipStack.length; i++) {
    clearTimeout(_tooltipStack[i]._hideTimer);
    _tooltipStack[i].classList.remove("visible");
    _tooltipStack[i].style.pointerEvents = "none";
  }
  _tooltipCooldown = Date.now();
}

// Hide tooltip on any navigation (click or popstate)
document.addEventListener("click", function () { hideTooltip(); });
window.addEventListener("popstate", function () { hideTooltip(); });

// Print with diagrams prepared for light/static rendering
function printPost() {
  var el = document.getElementById("post-content");
  if (!el) { window.print(); return; }

  // ---- Prepare TikZ diagrams: reset scaling and absolute positioning ----
  var tikzWrappers = el.querySelectorAll(".tikz-diagram");
  var savedTikz = [];
  tikzWrappers.forEach(function (wrapper) {
    var page = wrapper.querySelector(".page");
    var svg = page ? page.querySelector("svg") : null;
    savedTikz.push({
      wrapperCss: wrapper.style.cssText,
      pageCss: page ? page.style.cssText : "",
      svgCss: svg ? svg.style.cssText : ""
    });
    if (page) {
      page.style.cssText = "position: static; height: auto; width: auto;";
    }
    if (svg) {
      svg.style.position = "static";
    }
    wrapper.style.height = "auto";
  });

  function restoreTikz() {
    tikzWrappers.forEach(function (wrapper, i) {
      wrapper.style.cssText = savedTikz[i].wrapperCss;
      var page = wrapper.querySelector(".page");
      var svg = page ? page.querySelector("svg") : null;
      if (page) page.style.cssText = savedTikz[i].pageCss;
      if (svg) svg.style.cssText = savedTikz[i].svgCss;
    });
  }

  // ---- Prepare Mermaid diagrams: re-render in light theme ----
  var mermaidNodes = window.mermaid ? el.querySelectorAll(".mermaid[data-source]") : [];
  if (mermaidNodes.length > 0) {
    var config = getMermaidConfig("light");
    mermaid.initialize(config);
    mermaidNodes.forEach(function (node) {
      var source = node.getAttribute("data-source");
      if (source) {
        node.removeAttribute("data-processed");
        node.innerHTML = source;
      }
    });
    mermaid.run({ nodes: mermaidNodes }).then(function () {
      window.print();
      restoreTikz();
      // Restore mermaid to current theme
      var theme = document.documentElement.getAttribute("data-theme") || "light";
      mermaid.initialize(getMermaidConfig(theme));
      mermaidNodes.forEach(function (node) {
        var source = node.getAttribute("data-source");
        if (source) {
          node.removeAttribute("data-processed");
          node.innerHTML = source;
        }
      });
      mermaid.run({ nodes: mermaidNodes });
    });
  } else {
    window.print();
    restoreTikz();
  }
}

function setupTooltips(el, depth) {
    depth = depth || 0;
    var maxDepth = 4; // Prevent infinite nesting
    if (depth > maxDepth) return;

    function showTooltipWithContent(link, htmlContent, renderMath) {
      var tooltip = getTooltipAt(depth);
      tooltip.innerHTML = htmlContent;
      if (renderMath && window.renderMathInElement) {
        renderMathInElement(tooltip, {
          delimiters: [
            { left: "\\(", right: "\\)", display: false },
            { left: "\\[", right: "\\]", display: true },
          ],
          throwOnError: false,
        });
      }
      // Add pin button for blocks that have label metadata
      var blockLabel = link.getAttribute("data-block-label");
      if (blockLabel) {
        // Capture content before adding the pin button so it doesn't end up in previews
        var contentForPin = tooltip.innerHTML;
        var pinBtn = document.createElement("button");
        pinBtn.className = "tooltip-pin-btn";
        var pinned = isBlockPinned(blockLabel);
        pinBtn.textContent = pinned ? "unpin" : "pin";
        pinBtn.title = pinned ? "Unpin from sidebar" : "Pin to sidebar";
        pinBtn.addEventListener("click", function (e) {
          e.stopPropagation();
          if (isBlockPinned(blockLabel)) {
            removePinnedBlock(blockLabel);
            pinBtn.textContent = "pin";
            pinBtn.title = "Pin to sidebar";
          } else {
            addPinnedBlock({
              label: blockLabel,
              kind: link.getAttribute("data-block-kind") || "",
              title: link.getAttribute("data-block-title") || "",
              number: link.getAttribute("data-block-number") || "",
              preview: contentForPin,
              href: link.getAttribute("href") || ""
            });
            pinBtn.textContent = "unpin";
            pinBtn.title = "Unpin from sidebar";
          }
        });
        tooltip.appendChild(pinBtn);
      }
      positionTooltip(tooltip, link);
      // Enable nested hovers within this tooltip
      if (depth < maxDepth) {
        setupTooltips(tooltip, depth + 1);
      }
    }

    // Shared: bind mouseenter/mouseleave tooltip handlers to a set of links
    function bindTooltip(links, contentFn) {
      links.forEach(function (link) {
        if (link._tooltipBound) return;
        link._tooltipBound = true;
        link.addEventListener("mouseenter", function () {
          if (depth === 0 && Date.now() - _tooltipCooldown < _tooltipCooldownDuration) return;
          clearTimeout(_tooltipShowTimer);
          _tooltipShowTimer = setTimeout(function () { contentFn(link); }, 150);
        });
        link.addEventListener("mouseleave", function () {
          clearTimeout(_tooltipShowTimer);
          scheduleHideTooltipAt(depth);
        });
      });
    }

    // Cross-references and auto-definitions
    bindTooltip(el.querySelectorAll("[data-preview]"), function (link) {
      var href = link.getAttribute("href");
      var blockId = href && href.startsWith("#") ? href.slice(1) : null;
      var blockEl = blockId ? document.getElementById(blockId) : null;
      if (blockEl && blockEl.classList.contains("labeled-block")) {
        var clone = blockEl.cloneNode(true);
        clone.querySelectorAll(".copy-btn, .block-pin-btn").forEach(function(b) { b.remove(); });
        showTooltipWithContent(link, clone.innerHTML, false);
      } else {
        var preview = link.getAttribute("data-preview");
        if (preview) showTooltipWithContent(link, markdownPreview(preview), true);
      }
    });

    // Internal post links
    bindTooltip(
      Array.from(el.querySelectorAll("a[data-post-title]")).filter(function (l) {
        return !l.hasAttribute("data-preview");
      }),
      function (link) {
        var title = link.getAttribute("data-post-title");
        if (!title) return;
        var desc = link.getAttribute("data-post-desc");
        var tags = link.getAttribute("data-post-tags");
        var series = link.getAttribute("data-post-series");
        var html = "<strong>" + title + "</strong>";
        if (desc) html += "<p>" + desc + "</p>";
        if (series) html += '<p style="color:var(--accent);font-size:0.8rem">' + series + "</p>";
        if (tags) html += '<p style="font-size:0.75rem;color:var(--text-muted)">' + tags.split(", ").map(function(t) { return "#" + t; }).join(" ") + "</p>";
        showTooltipWithContent(link, html, false);
      }
    );

    // Internal non-post links (e.g. /about, /projects)
    bindTooltip(
      Array.from(el.querySelectorAll("a[href^='/']")).filter(function (l) {
        return !l.hasAttribute("data-post-title") && !l.hasAttribute("data-preview")
          && !l.getAttribute("href").startsWith("/blog/");
      }),
      function (link) {
        var href = link.getAttribute("href");
        if (!href) return;
        var linkText = link.textContent.trim();
        showTooltipWithContent(link, '<div class="ext-preview"><div class="ext-title">' + linkText + '</div><div class="ext-path">' + href + '</div></div>', false);
      }
    );

    // External links
    bindTooltip(
      Array.from(el.querySelectorAll("a[href^='http']")).filter(function (l) {
        return !l.hasAttribute("data-post-title") && !l.hasAttribute("data-preview");
      }),
      function (link) {
        var href = link.getAttribute("href");
        if (!href) return;
        try { var url = new URL(href); } catch (e) { return; }
        var favicon = "https://www.google.com/s2/favicons?domain=" + url.hostname + "&sz=32";
        var linkText = link.textContent.trim();
        var displayText = linkText !== href ? linkText : "";
        var path = url.pathname + url.search + url.hash;
        if (path.length > 60) path = path.slice(0, 57) + "...";

        var og = _ogMetadata && _ogMetadata[href];
        var html = '<div class="ext-preview">';
        html += '<div class="ext-preview-header">';
        html += '<img class="ext-favicon" src="' + favicon + '" width="16" height="16" alt="" />';
        html += '<span class="ext-domain">' + url.hostname + '</span></div>';
        if (og && og.title) {
          html += '<div class="ext-title">' + escapeHtml(og.title) + '</div>';
        } else if (displayText && displayText !== url.hostname) {
          html += '<div class="ext-title">' + displayText + '</div>';
        }
        if (og && og.description) {
          html += '<div class="ext-desc">' + escapeHtml(og.description) + '</div>';
        }
        if (og && og.image) {
          html += '<img class="ext-image" src="' + escapeHtml(og.image) + '" alt="" />';
        }
        if (path !== "/") html += '<div class="ext-path">' + path + '</div>';
        html += '</div>';
        showTooltipWithContent(link, html, false);
      }
    );
}

function markdownPreview(md) {
  // Step 1: Extract math/code blocks into placeholders so text transforms don't mangle them
  var placeholders = [];
  function placeholder(content) {
    var id = "\x00PLACEHOLDER_" + placeholders.length + "\x00";
    placeholders.push(content);
    return id;
  }

  var s = md;

  // Fenced code blocks
  s = s.replace(/```(\w*)\n?([\s\S]*?)```/g, function(_, lang, code) {
    return placeholder("<pre><code>" + code + "</code></pre>");
  });

  // LaTeX environments (align, gather, equation, etc.) — must come before $$ matching
  s = s.replace(/\\begin\{(align|align\*|multline|split|gather|gather\*|equation|equation\*)\}([\s\S]*?)\\end\{\1\}/g, function(_, env, body) {
    if (env === "equation" || env === "equation*") {
      return placeholder("\\[" + body + "\\]");
    }
    return placeholder("\\[\\begin{" + env + "}" + body + "\\end{" + env + "}\\]");
  });

  // Display math $$...$$
  s = s.replace(/\$\$([\s\S]+?)\$\$/g, function(_, body) {
    return placeholder("\\[" + body + "\\]");
  });

  // Inline code `...` (before inline math to avoid conflicts)
  s = s.replace(/`(.+?)`/g, function(_, code) {
    return placeholder("<code>" + code + "</code>");
  });

  // Inline math $...$
  s = s.replace(/\$(.+?)\$/g, function(_, body) {
    return placeholder("\\(" + body + "\\)");
  });

  // Step 2: Text transforms on the remaining markdown
  s = s
    .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
    .replace(/\*(.+?)\*/g, "<em>$1</em>")
    .replace(/^(\d+)\.\s+(.+)$/gm, "<li>$2</li>")
    .replace(/^[-*]\s+(.+)$/gm, "<li>$1</li>")
    .replace(/(<li>.*<\/li>\n?)+/g, function(match) {
      return "<ul>" + match + "</ul>";
    })
    .replace(/\n\n/g, "</p><p>")
    .replace(/\n/g, " ")
    .replace(/^/, "<p>")
    .replace(/$/, "</p>")
    .replace(/<p>\s*<\/p>/g, "")
    .replace(/<p>\s*(<ul>)/g, "$1")
    .replace(/(<\/ul>)\s*<\/p>/g, "$1");

  // Step 3: Restore placeholders
  for (var i = 0; i < placeholders.length; i++) {
    s = s.replace("\x00PLACEHOLDER_" + i + "\x00", placeholders[i]);
  }

  return s;
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

    function onMouseMove(e) {
      state.tx += e.clientX - state.lastX;
      state.ty += e.clientY - state.lastY;
      state.lastX = e.clientX;
      state.lastY = e.clientY;
      applyTransform();
    }

    container.addEventListener("mousedown", function (e) {
      if (e.button !== 0 || state.dragging) return;
      state.dragging = true;
      state.lastX = e.clientX;
      state.lastY = e.clientY;
      container.style.cursor = "grabbing";
      window.addEventListener("mousemove", onMouseMove);
      window.addEventListener("mouseup", function handler() {
        state.dragging = false;
        container.style.cursor = "grab";
        window.removeEventListener("mousemove", onMouseMove);
        window.removeEventListener("mouseup", handler);
      });
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

// ============================================================
// Pin buttons on labeled blocks (hover to reveal)
// ============================================================

function setupBlockPinButtons(el) {
  el.querySelectorAll(".labeled-block").forEach(function (block) {
    if (block.querySelector(".block-pin-btn")) return;
    var id = block.getAttribute("id");
    if (!id) return;

    var header = block.querySelector(".labeled-block-header");
    if (!header) return;

    // Extract block metadata from the rendered header
    var headerText = header.textContent.trim();
    var kind = "";
    var number = "";
    var title = "";
    var strong = header.querySelector("strong");
    if (strong) {
      var parts = strong.textContent.replace(/\.$/, "").split(" ");
      kind = (parts[0] || "").toLowerCase();
      number = parts.slice(1).join(" ");
    }
    // Title is in parentheses after the strong tag
    var fullText = header.textContent;
    var parenMatch = fullText.match(/\(([^)]+)\)/);
    if (parenMatch) title = parenMatch[1];

    // For proof blocks, find the associated theorem/lemma from the previous sibling
    if (kind === "proof" && !title) {
      var prev = block.previousElementSibling;
      if (prev && prev.classList.contains("labeled-block") && !prev.classList.contains("proof")) {
        var prevHeader = prev.querySelector(".labeled-block-header");
        if (prevHeader) {
          var prevStrong = prevHeader.querySelector("strong");
          var prevKind = prevStrong ? prevStrong.textContent.replace(/\.\s*$/, "").trim() : "";
          var prevParen = prevHeader.textContent.match(/\(([^)]+)\)/);
          var prevTitle = prevParen ? prevParen[1] : "";
          title = prevTitle ? prevKind + ", " + prevTitle : prevKind;
        }
      }
    }

    // Get preview content (the block body text, not HTML)
    var contentEl = block.querySelector(".labeled-block-content");
    var preview = contentEl ? contentEl.textContent.trim().slice(0, 500) : "";

    var btn = document.createElement("button");
    btn.className = "block-pin-btn";
    var pinned = isBlockPinned(id);
    btn.textContent = pinned ? "unpin" : "pin";
    btn.title = pinned ? "Unpin from sidebar" : "Pin to sidebar";
    btn.addEventListener("click", function (e) {
      e.stopPropagation();
      if (isBlockPinned(id)) {
        removePinnedBlock(id);
        btn.textContent = "pin";
        btn.title = "Pin to sidebar";
      } else {
        // Get the raw markdown preview from the block's content
        var rawPreview = contentEl ? contentEl.innerHTML : "";
        addPinnedBlock({
          label: id,
          kind: kind,
          title: title,
          number: number,
          preview: rawPreview,
          href: window.location.pathname + "#" + id
        });
        btn.textContent = "unpin";
        btn.title = "Unpin from sidebar";
      }
    });
    // Sync button state when pins change from the panel side
    window.addEventListener("pinned-blocks-changed", function () {
      var nowPinned = isBlockPinned(id);
      btn.textContent = nowPinned ? "unpin" : "pin";
      btn.title = nowPinned ? "Unpin from sidebar" : "Pin to sidebar";
    });

    header.style.position = "relative";
    header.appendChild(btn);
  });
}

// ============================================================
// Pinned blocks (localStorage persistence)
// ============================================================

var _PINNED_KEY = "pinned-blocks";

function getPinnedBlocks() {
  try {
    return JSON.parse(localStorage.getItem(_PINNED_KEY)) || [];
  } catch (e) {
    return [];
  }
}

function isBlockPinned(label) {
  return getPinnedBlocks().some(function (b) { return b.label === label; });
}

function addPinnedBlock(block) {
  var blocks = getPinnedBlocks();
  if (blocks.some(function (b) { return b.label === block.label; })) return;
  blocks.push(block);
  localStorage.setItem(_PINNED_KEY, JSON.stringify(blocks));
  window.dispatchEvent(new CustomEvent("pinned-blocks-changed"));
}

function removePinnedBlock(label) {
  var blocks = getPinnedBlocks().filter(function (b) { return b.label !== label; });
  localStorage.setItem(_PINNED_KEY, JSON.stringify(blocks));
  window.dispatchEvent(new CustomEvent("pinned-blocks-changed"));
}
