export const THEME_STORAGE_KEY = "uniconvert-theme"

export function getPreferredDarkTheme() {
  const savedTheme = localStorage.getItem(THEME_STORAGE_KEY)

  if (savedTheme === "dark" || savedTheme === "light") {
    return savedTheme === "dark"
  }

  return window.matchMedia("(prefers-color-scheme: dark)").matches
}

export function applyTheme(isDark: boolean) {
  document.documentElement.classList.toggle("dark", isDark)
}

export function saveTheme(isDark: boolean) {
  localStorage.setItem(THEME_STORAGE_KEY, isDark ? "dark" : "light")
}
