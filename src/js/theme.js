function getMermaidConfig(theme) {
  var isDark = theme === "dark";
  return {
    startOnLoad: false,
    theme: "base",
    fontFamily: '"0xProto Nerd Font", "0xProto", monospace',
    themeVariables: {
      // Match site CSS variables exactly
      primaryColor: isDark ? "#111820" : "#f6f8fa",
      primaryTextColor: isDark ? "#c9d1d9" : "#24292e",
      primaryBorderColor: isDark ? "#30363d" : "#e1e4e8",
      lineColor: isDark ? "#8b949e" : "#6a737d",
      secondaryColor: isDark ? "#080c12" : "#ffffff",
      tertiaryColor: isDark ? "#080c12" : "#ffffff",
      mainBkg: isDark ? "#111820" : "#f6f8fa",
      nodeBorder: isDark ? "#30363d" : "#e1e4e8",
      clusterBkg: isDark ? "#080c12" : "#ffffff",
      titleColor: isDark ? "#c9d1d9" : "#24292e",
      edgeLabelBackground: isDark ? "#080c12" : "#ffffff",
      noteTextColor: isDark ? "#c9d1d9" : "#24292e",
      noteBkgColor: isDark ? "#111820" : "#f6f8fa",
      noteBorderColor: isDark ? "#30363d" : "#e1e4e8",
    },
  };
}
