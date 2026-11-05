import { useEffect, useRef, useState } from "react"
import { convertFileSrc, invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { LoaderCircleIcon, PauseIcon, PlayIcon, RotateCcwIcon, ScissorsIcon } from "lucide-react"
import { Button } from "@/shared/ui/button"
import { Input } from "@/shared/ui/input"
import { Slider } from "@/shared/ui/slider"
import { Progress } from "@/shared/ui/progress"
import { formatAudioTime, parseAudioTime, type AudioTrim } from "../model/time"

type Preview = { path: string; durationSeconds: number; peaks: number[] }
type Props = {
  path?: string
  file?: File
  disabled?: boolean
  onChange: (trim: AudioTrim | null, valid: boolean) => void
}

function errorMessage(error: unknown) {
  if (error === "Cancelled") return "Подготовка аудио отменена."
  if (typeof error === "string") return error
  if (error instanceof Error) return error.message
  if (error && typeof error === "object") {
    const message = Object.values(error).find((value) => typeof value === "string")
    if (message) return String(message)
  }
  return "Не удалось подготовить аудио для прослушивания."
}

export function AudioTrimmer({ path, file, disabled, onChange }: Props) {
  const audioRef = useRef<HTMLAudioElement>(null)
  const jobRef = useRef<string | null>(null)
  const [attempt, setAttempt] = useState(0)
  const [source, setSource] = useState("")
  const [duration, setDuration] = useState(0)
  const [peaks, setPeaks] = useState<number[]>([])
  const [range, setRange] = useState<AudioTrim>({ start: 0, end: 0 })
  const [draft, setDraft] = useState({ start: "00:00.000", end: "00:00.000" })
  const [valid, setValid] = useState(true)
  const [position, setPosition] = useState(0)
  const [playing, setPlaying] = useState(false)
  const [loading, setLoading] = useState(true)
  const [progress, setProgress] = useState(0)
  const [error, setError] = useState("")

  useEffect(() => {
    let disposed = false
    let started = false
    let unlisten: (() => void) | undefined
    let objectUrl: string | undefined
    let context: AudioContext | undefined
    const audio = audioRef.current
    const jobId = crypto.randomUUID()
    jobRef.current = jobId
    setLoading(true)
    setError("")
    setSource("")
    setProgress(0)
    onChange(null, true)

    async function prepare() {
      try {
        let nextSource: string
        let nextDuration: number
        let nextPeaks: number[]
        if (path) {
          unlisten = await listen<{ jobId: string; percent: number }>("audio-preview-progress", ({ payload }) => {
            if (!disposed && payload.jobId === jobId) setProgress(payload.percent)
          })
          if (disposed) { unlisten(); return }
          started = true
          const preview = await invoke<Preview>("prepare_audio_preview", { input: path, jobId })
          if (disposed) {
            await invoke("release_audio_preview", { jobId })
            return
          }
          nextSource = convertFileSrc(preview.path)
          nextDuration = preview.durationSeconds
          nextPeaks = preview.peaks
        } else if (file) {
          context = new AudioContext()
          const buffer = await context.decodeAudioData(await file.arrayBuffer())
          if (disposed) return
          nextDuration = buffer.duration
          nextPeaks = Array.from({ length: 1200 }, () => 0)
          for (let channel = 0; channel < buffer.numberOfChannels; channel++) {
            const samples = buffer.getChannelData(channel)
            for (let i = 0; i < samples.length; i++) {
              const bin = Math.floor(i * nextPeaks.length / samples.length)
              nextPeaks[bin] = Math.max(nextPeaks[bin], Math.abs(samples[i]))
            }
          }
          objectUrl = URL.createObjectURL(file)
          nextSource = objectUrl
        } else {
          throw new Error("Выберите аудиофайл.")
        }
        if (!Number.isFinite(nextDuration) || nextDuration <= 0) throw new Error("Аудиозапись пуста.")
        // Millisecond boundaries stay within the actual decoded duration.
        nextDuration = Math.floor(nextDuration * 1000) / 1000
        setDuration(nextDuration)
        setPeaks(nextPeaks)
        setRange({ start: 0, end: nextDuration })
        setDraft({ start: formatAudioTime(0), end: formatAudioTime(nextDuration) })
        setValid(true)
        setPosition(0)
        setSource(nextSource)
      } catch (error) {
        if (!disposed) setError(errorMessage(error))
      } finally {
        if (context && context.state !== "closed") void context.close()
        if (!disposed) setLoading(false)
      }
    }
    void prepare()
    return () => {
      disposed = true
      unlisten?.()
      audio?.pause()
      audio?.removeAttribute("src")
      audio?.load()
      if (objectUrl) URL.revokeObjectURL(objectUrl)
      if (path && started) {
        void invoke("cancel_convert", { jobId }).catch(() => {})
        void invoke("release_audio_preview", { jobId }).catch(() => {})
      }
    }
  }, [path, file, attempt, onChange])

  useEffect(() => {
    if (disabled || !valid) audioRef.current?.pause()
  }, [disabled, valid])

  useEffect(() => {
    if (!playing) return
    let frame = 0
    const tick = () => {
      const audio = audioRef.current
      if (!audio) return
      if (audio.currentTime >= range.end) {
        audio.pause()
        audio.currentTime = range.end
      }
      setPosition(audio.currentTime)
      frame = requestAnimationFrame(tick)
    }
    frame = requestAnimationFrame(tick)
    return () => cancelAnimationFrame(frame)
  }, [playing, range.end])

  function seek(seconds: number) {
    const value = Math.max(range.start, Math.min(range.end, seconds))
    if (audioRef.current) audioRef.current.currentTime = value
    setPosition(value)
  }

  function updateRange(next: AudioTrim, updateDraft = true) {
    audioRef.current?.pause()
    setRange(next)
    setValid(true)
    if (updateDraft) setDraft({ start: formatAudioTime(next.start), end: formatAudioTime(next.end) })
    if (audioRef.current) audioRef.current.currentTime = next.start
    setPosition(next.start)
    onChange(next.start === 0 && next.end === duration ? null : next, true)
  }

  function editTime(edge: "start" | "end", text: string) {
    const nextDraft = { ...draft, [edge]: text }
    setDraft(nextDraft)
    const start = parseAudioTime(nextDraft.start)
    const end = parseAudioTime(nextDraft.end)
    if (start === null || end === null || end <= start || end > duration) {
      setValid(false)
      onChange(null, false)
      return
    }
    updateRange({ start, end }, false)
  }

  async function togglePlayback() {
    const audio = audioRef.current
    if (!audio) return
    if (!audio.paused) { audio.pause(); return }
    if (audio.currentTime < range.start || audio.currentTime >= range.end - 0.001) seek(range.start)
    try { await audio.play() } catch { setError("Не удалось воспроизвести аудио. Попробуйте подготовить его заново.") }
  }

  return (
    <section className="space-y-5 rounded-xl border bg-card p-5 text-card-foreground shadow-xs" aria-labelledby="audio-trim-title">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 id="audio-trim-title" className="flex items-center gap-2 text-sm font-medium"><ScissorsIcon className="size-4" />Прослушивание и обрезка</h2>
          <p className="mt-1 text-xs text-muted-foreground">Выберите фрагмент, который хотите сохранить.</p>
        </div>
        {!loading && !error && <Button variant="ghost" size="sm" disabled={disabled} onClick={() => updateRange({ start: 0, end: duration })}><RotateCcwIcon />Весь файл</Button>}
      </div>
      <audio ref={audioRef} src={source || undefined} preload="auto"
        onPlay={() => setPlaying(true)} onPause={() => setPlaying(false)} onEnded={() => setPlaying(false)}
        onError={() => { if (source) setError("Не удалось воспроизвести аудио. Попробуйте ещё раз.") }} />
      {loading ? (
        <div className="space-y-3" role="status">
          <div className="flex items-center gap-2 text-sm text-muted-foreground"><LoaderCircleIcon className="size-4 animate-spin" />Подготовка аудио… {Math.round(progress)}%</div>
          <Progress value={progress} aria-label="Подготовка аудио" />
          {path && <Button variant="outline" size="sm" onClick={() => {
            if (jobRef.current) void invoke("cancel_convert", { jobId: jobRef.current }).catch(() => {})
          }}>Отменить подготовку</Button>}
        </div>
      ) : error ? (
        <div className="space-y-3"><p role="alert" className="text-sm text-destructive">{error}</p><Button variant="outline" disabled={disabled} onClick={() => setAttempt((value) => value + 1)}>Попробовать снова</Button></div>
      ) : (
        <fieldset disabled={disabled} className="min-w-0 space-y-5 disabled:opacity-60">
          <div className="space-y-3">
            <div className="relative h-28 overflow-hidden rounded-lg bg-muted/40">
              <svg viewBox="0 0 1200 100" preserveAspectRatio="none" aria-hidden="true" className="absolute inset-0 size-full text-primary">
                {peaks.map((peak, index) => <line key={index} x1={index} x2={index} y1={50 - Math.max(1, peak * 46)} y2={50 + Math.max(1, peak * 46)} stroke="currentColor" strokeWidth="1" />)}
              </svg>
              <div className="pointer-events-none absolute inset-y-0 left-0 bg-background/75" style={{ width: `${range.start / duration * 100}%` }} />
              <div className="pointer-events-none absolute inset-y-0 right-0 bg-background/75" style={{ width: `${(duration - range.end) / duration * 100}%` }} />
              <div className="pointer-events-none absolute inset-y-0 border-x-2 border-primary bg-primary/5" style={{ left: `${range.start / duration * 100}%`, width: `${(range.end - range.start) / duration * 100}%` }} />
              <div className="pointer-events-none absolute inset-y-0 w-0.5 bg-foreground" style={{ left: `${Math.min(99.8, position / duration * 100)}%` }} />
              <button type="button" className="absolute inset-0 cursor-crosshair focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset outline-none" aria-label="Перемотка по звуковой волне. Стрелки — одна секунда."
                onClick={(event) => { const bounds = event.currentTarget.getBoundingClientRect(); seek((event.clientX - bounds.left) / bounds.width * duration) }}
                onKeyDown={(event) => { if (["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) { event.preventDefault(); seek(event.key === "Home" ? range.start : event.key === "End" ? range.end : position + (event.key === "ArrowLeft" ? -1 : 1)) } }} />
            </div>
            <Slider min={0} max={duration} step={0.001} minStepsBetweenValues={1} thumbCollisionBehavior="none"
              value={[range.start, range.end]} disabled={disabled} thumbLabels={["Начало фрагмента", "Конец фрагмента"]}
              onValueChange={(values) => { if (Array.isArray(values) && values[1] > values[0]) updateRange({ start: values[0], end: values[1] }) }} />
            <div className="flex justify-between text-xs tabular-nums text-muted-foreground"><span>00:00</span><span>{formatAudioTime(duration)}</span></div>
          </div>
          <div className="flex flex-wrap items-center gap-3">
            <Button disabled={disabled || !valid} onClick={() => void togglePlayback()} aria-label={playing ? "Пауза" : "Прослушать фрагмент"}>
              {playing ? <PauseIcon /> : <PlayIcon />}{playing ? "Пауза" : "Прослушать"}
            </Button>
            <span className="text-sm tabular-nums text-muted-foreground">{formatAudioTime(position)} / {formatAudioTime(duration)}</span>
            <span className="ml-auto rounded-md bg-muted px-2 py-1 text-xs tabular-nums">Фрагмент: {formatAudioTime(range.end - range.start)}</span>
          </div>
          <div className="grid gap-4 sm:grid-cols-2">
            <label className="space-y-2 text-sm"><span>Начало</span><Input aria-label="Начало, время" aria-invalid={!valid} aria-describedby="audio-time-help" value={draft.start} onChange={(event) => editTime("start", event.target.value)} className="tabular-nums" /></label>
            <label className="space-y-2 text-sm"><span>Конец</span><Input aria-label="Конец, время" aria-invalid={!valid} aria-describedby="audio-time-help" value={draft.end} onChange={(event) => editTime("end", event.target.value)} className="tabular-nums" /></label>
          </div>
          <p id="audio-time-help" className="text-xs text-muted-foreground">Время: мм:сс.ммм, чч:мм:сс.ммм или секунды. Прослушивание исходного звука, без настроек конвертации.</p>
          {!valid && <p role="alert" className="text-sm text-destructive">Укажите корректное время: конец позже начала и в пределах записи.</p>}
        </fieldset>
      )}
    </section>
  )
}
