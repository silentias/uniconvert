import { useEffect, useState } from "react"

import { applyTheme, getPreferredDarkTheme, saveTheme } from "@/shared/lib/theme"
import { Switch } from "@/shared/ui/switch"

export function ThemeSwitch() {
  const [isDark, setIsDark] = useState(getPreferredDarkTheme)

  useEffect(() => {
    applyTheme(isDark)
    saveTheme(isDark)
  }, [isDark])

  return (
    <Switch
      checked={isDark}
      onCheckedChange={setIsDark}
      aria-label="Переключить тёмную тему"
    />
  )
}
