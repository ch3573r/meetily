"use client"

export type ThemePreference = "light" | "dark" | "system"
export type ResolvedTheme = "light" | "dark"

export const THEME_STORAGE_KEY = "clawscribe.theme"

export const themePreferences: ThemePreference[] = ["light", "dark", "system"]

const isThemePreference = (value: string | null): value is ThemePreference =>
  value === "light" || value === "dark" || value === "system"

export function getStoredThemePreference(): ThemePreference {
  if (typeof window === "undefined") return "system"

  try {
    const stored = window.localStorage.getItem(THEME_STORAGE_KEY)
    return isThemePreference(stored) ? stored : "system"
  } catch {
    return "system"
  }
}

export function getSystemTheme(): ResolvedTheme {
  if (typeof window === "undefined") return "light"

  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"
}

export function resolveThemePreference(preference: ThemePreference): ResolvedTheme {
  return preference === "system" ? getSystemTheme() : preference
}

export function applyThemePreference(preference: ThemePreference): ResolvedTheme {
  const resolvedTheme = resolveThemePreference(preference)

  if (typeof document !== "undefined") {
    const root = document.documentElement
    root.classList.toggle("dark", resolvedTheme === "dark")
    root.dataset.theme = resolvedTheme
    root.dataset.themePreference = preference
    root.style.colorScheme = resolvedTheme
  }

  return resolvedTheme
}

export function setThemePreference(preference: ThemePreference): ResolvedTheme {
  if (typeof window !== "undefined") {
    try {
      window.localStorage.setItem(THEME_STORAGE_KEY, preference)
    } catch {
      // Theme changes should still apply even if storage is unavailable.
    }
  }

  return applyThemePreference(preference)
}

export function subscribeToSystemTheme(callback: () => void): () => void {
  if (typeof window === "undefined") return () => undefined

  const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)")
  mediaQuery.addEventListener("change", callback)

  return () => mediaQuery.removeEventListener("change", callback)
}
