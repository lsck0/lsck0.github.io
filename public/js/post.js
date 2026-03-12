function renderPost() {
  requestAnimationFrame(function () {
    requestAnimationFrame(function () {
      var el = document.getElementById("post-content");
      if (!el) return;

      // Convert tikz pre elements to script elements, wrapped in a
      // centering container. tikzjax replaces the <script> with an SVG
      // but the wrapper div survives.
      var hasTikz = false;
      el.querySelectorAll("pre.tikz-src").forEach(function (pre) {
        hasTikz = true;
        var wrapper = document.createElement("div");
        wrapper.className = "tikz-diagram";

        var s = document.createElement("script");
        s.type = "text/tikz";
        var libs = pre.getAttribute("data-libs");
        if (libs) s.setAttribute("data-tikz-libraries", libs);
        s.textContent = pre.textContent;

        wrapper.appendChild(s);
        pre.replaceWith(wrapper);
      });

      // Render tikz
      if (hasTikz && window._tikzjaxRender) {
        window._tikzjaxRender();

        // tikzjax creates: .page (height:0, width:100%, position:relative)
        //                 > svg  (position:absolute, top:0, left:0)
        // The SVG is out of flow, so we read its natural size, apply
        // transform:scale on .page for visual enlargement, and set
        // explicit dimensions on the .tikz-diagram wrapper so surrounding
        // content reserves the correct space.
        var TIKZ_SCALE = 1.8;
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

            // Scale .page visually via transform (doesn't affect layout)
            page.style.setProperty("transform", "scale(" + TIKZ_SCALE + ")", "important");
            page.style.setProperty("transform-origin", "top left", "important");
            // Give .page its natural dimensions so transform has something to scale
            page.style.setProperty("width", w, "important");
            page.style.setProperty("height", h, "important");

            // Reserve the scaled space on the wrapper so content below doesn't overlap
            wrapper.style.width = (widthPt * TIKZ_SCALE) + "pt";
            wrapper.style.height = (heightPt * TIKZ_SCALE) + "pt";
            wrapper.dataset.sizeFixed = "true";
          });
        }, 500);
        setTimeout(function () {
          clearInterval(tikzPoll);
        }, 15000);
      }

      // KaTeX math rendering
      if (window.renderMathInElement) {
        renderMathInElement(el, {
          delimiters: [
            { left: "\\(", right: "\\)", display: false },
            { left: "\\[", right: "\\]", display: true },
          ],
          throwOnError: false,
        });
      }

      // Mermaid diagrams
      if (window.mermaid) {
        var theme =
          document.documentElement.getAttribute("data-theme") || "light";
        var config = getMermaidConfig(theme);
        mermaid.initialize(config);
        var nodes = el.querySelectorAll(".mermaid");
        if (nodes.length > 0) {
          nodes.forEach(function (node) {
            node.setAttribute("data-source", node.textContent);
          });
          mermaid.run({ nodes: nodes });
        }
      }

      // Syntax highlighting
      if (window.Prism) {
        Prism.highlightAllUnder(el);
      }
    });
  });
}
