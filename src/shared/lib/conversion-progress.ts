export type ProgressSample = { seconds: number; percent: number }

export function estimateProgress(samples: ProgressSample[], now: number) {
  const first = samples[0]
  const last = samples[samples.length - 1]
  if (!first || !last || now - last.seconds > 10 || last.percent >= 99.5) return null
  const seconds = last.seconds - first.seconds
  const delta = last.percent - first.percent
  if (seconds < 2 || delta <= 0 || last.percent < 1) return null
  const rate = delta / seconds
  return { percentPerMinute: rate * 60, remainingSeconds: (100 - last.percent) / rate }
}

export function formatElapsed(seconds: number) {
  const total = Math.max(0, Math.floor(seconds))
  const hours = Math.floor(total / 3600)
  const minutes = Math.floor(total / 60) % 60
  const rest = String(total % 60).padStart(2, "0")
  return hours ? `${hours}:${String(minutes).padStart(2, "0")}:${rest}` : `${minutes}:${rest}`
}
