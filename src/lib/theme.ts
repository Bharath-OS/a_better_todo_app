export function applyTheme(theme: string) {
  const root = document.documentElement;
  if (theme === "dark") {
    root.dataset.theme = "dark";
    return;
  }
  if (theme === "light") {
    delete root.dataset.theme;
    return;
  }
  const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  if (prefersDark) root.dataset.theme = "dark";
  else delete root.dataset.theme;
}
