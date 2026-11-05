export type AudioTrim = { start: number; end: number }

export function formatAudioTime(seconds: number): string {
  const milliseconds = Math.max(0, Math.round(seconds * 1000))
  const minutes = Math.floor(milliseconds / 60000)
  const remainder = ((milliseconds % 60000) / 1000).toFixed(3).padStart(6, "0")
  return `${String(minutes).padStart(2, "0")}:${remainder}`
}

export function parseAudioTime(text: string): number | null {
  const value = text.trim().replace(",", ".")
  if (!/^\d+(?::\d{1,2}){0,2}(?:\.\d{1,3})?$/.test(value)) return null
  const parts = value.split(":").map(Number)
  if (parts.slice(1).some((part) => part >= 60)) return null
  const seconds = parts.reduce((total, part) => total * 60 + part, 0)
  return Number.isFinite(seconds) ? seconds : null
}
