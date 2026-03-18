function getMermaidConfig(theme) {
  var isDark = theme === "dark";
  return {
    startOnLoad: false,
    theme: "base",
    fontFamily: '"0xProto Nerd Font", "0xProto", monospace',
    themeVariables: {
      primaryColor: isDark ? "#1c2333" : "#e8eef4",
      primaryTextColor: isDark ? "#c9d1d9" : "#24292e",
      primaryBorderColor: isDark ? "#30363d" : "#c8d1da",
      lineColor: isDark ? "#8b949e" : "#6a737d",
      secondaryColor: isDark ? "#161b22" : "#f6f8fa",
      tertiaryColor: isDark ? "#0a0e14" : "#ffffff",
      mainBkg: isDark ? "#1c2333" : "#e8eef4",
      nodeBorder: isDark ? "#30363d" : "#c8d1da",
      clusterBkg: isDark ? "#161b22" : "#f6f8fa",
      titleColor: isDark ? "#c9d1d9" : "#24292e",
      edgeLabelBackground: isDark ? "#161b22" : "#f6f8fa",
      noteTextColor: isDark ? "#c9d1d9" : "#24292e",
      noteBkgColor: isDark ? "#161b22" : "#f6f8fa",
      noteBorderColor: isDark ? "#30363d" : "#c8d1da",
    },
  };
}
