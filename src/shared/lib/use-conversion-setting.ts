import { useEffect, useState } from "react"

export function useConversionSetting<T extends string | number | boolean>(
  name: string,
  fallback: T,
  validate: (value: T) => boolean = () => true,
) {
  const key = `uniconvert.conversion-settings.v1.${name}`
  const [value, setValue] = useState<T>(() => {
    try {
      const raw = localStorage.getItem(key)
      if (raw === null) return fallback
      const saved: unknown = JSON.parse(raw)
      return typeof saved === typeof fallback && validate(saved as T) ? saved as T : fallback
    } catch { return fallback }
  })
  useEffect(() => {
    try { localStorage.setItem(key, JSON.stringify(value)) }
    catch { /* Settings remain usable when browser storage is unavailable. */ }
  }, [key, value])
  return [value, setValue] as const
}

export const within = (min: number, max: number) => (value: number) =>
  Number.isFinite(value) && value >= min && value <= max

export const oneOf = (values: readonly string[]) => (value: string) => values.includes(value)
