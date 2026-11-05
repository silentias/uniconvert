import { useCallback, useEffect, useRef, useState } from "react"
import { CircleAlertIcon, CircleCheckIcon, LoaderCircleIcon } from "lucide-react"
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"

import { getDefaultOutputDirectory } from "@/entities/app-settings"
import { addConversionRecord } from "@/entities/conversion-history"
import {
  audioOutputFormats,
  detectMediaFile,
  imageOutputFormats,
  videoOutputFormats,
} from "@/entities/media-file"
import { OutputDirectoryPicker } from "@/features/select-output-directory"
import { AudioTrimmer, type AudioTrim } from "@/features/trim-audio"
import {
  MediaFileUploader,
  type UploadedFile,
} from "@/features/upload-media-file"
import { Button } from "@/shared/ui/button"
import { Input } from "@/shared/ui/input"
import { Progress, ProgressLabel, ProgressValue } from "@/shared/ui/progress"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/shared/ui/tabs"
import { estimateProgress, formatElapsed, type ProgressSample } from "@/shared/lib/conversion-progress"
import { useConversionSetting, within, oneOf } from "@/shared/lib/use-conversion-setting"

type OutputEstimate = {
  estimatedBytes: number
  lowerBytes: number
  upperBytes: number
}

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} Б`
  const units = ["КБ", "МБ", "ГБ", "ТБ"]
  let value = bytes / 1024
  let unit = units[0]
  for (let index = 1; value >= 1024 && index < units.length; index += 1) {
    value /= 1024
    unit = units[index]
  }
  return `${value.toFixed(value >= 100 ? 0 : 1)} ${unit}`
}

export function ConversionPage() {
  const [historyWarning, setHistoryWarning] = useState("")
  const [outputDirectory, setOutputDirectory] = useState(
    getDefaultOutputDirectory,
  )
  const [uploadedFile, setUploadedFile] = useState<UploadedFile | null>(null)
  const [imageFormat, setImageFormat] = useConversionSetting<string>("imageFormat", imageOutputFormats[0], oneOf(imageOutputFormats))
  const [audioFormat, setAudioFormat] = useConversionSetting<string>("audioFormat", audioOutputFormats[0], oneOf(audioOutputFormats))
  const [videoFormat, setVideoFormat] = useConversionSetting<string>("videoFormat", videoOutputFormats[0], oneOf(videoOutputFormats))
  const [quality, setQuality] = useConversionSetting<number>("quality", 80, within(0, 100))
  const [stripMetadata, setStripMetadata] = useConversionSetting<boolean>("stripMetadata", false)
  const [audioBitrate, setAudioBitrate] = useConversionSetting<string>("audioBitrate", "192", oneOf(["auto", "64", "96", "128", "160", "192", "256", "320"]))
  const [audioSampleRate, setAudioSampleRate] = useConversionSetting<string>("audioSampleRate", "44100", oneOf(["auto", "22050", "44100", "48000", "96000"]))
  const [audioChannels, setAudioChannels] = useConversionSetting<string>("audioChannels", "source", oneOf(["source", "1", "2"]))
  const [audioVolume, setAudioVolume] = useConversionSetting<number>("audioVolume", 0, within(-60, 30))
  const [audioTempo, setAudioTempo] = useConversionSetting<number>("audioTempo", 1, within(0.5, 2))
  const [audioPitch, setAudioPitch] = useConversionSetting<number>("audioPitch", 0, within(-12, 12))
  const [normalizeAudio, setNormalizeAudio] = useConversionSetting<boolean>("normalizeAudio", false)
  const [audioTargetLufs, setAudioTargetLufs] = useConversionSetting<string>("audioTargetLufs", "-14", oneOf(["-23", "-16", "-14"]))
  const [audioTrim, setAudioTrim] = useState<AudioTrim | null>(null)
  const [audioTrimValid, setAudioTrimValid] = useState(true)
  const [videoCodec, setVideoCodec] = useConversionSetting<string>("videoCodec", "h264", oneOf(["h264", "h265", "vp9", "av1"]))
  const [videoCrf, setVideoCrf] = useConversionSetting<number>("videoCrf", 23, within(0, 51))
  const [videoHeight, setVideoHeight] = useConversionSetting<string>("videoHeight", "auto", oneOf(["auto", "2160", "1440", "1080", "720", "480"]))
  const [videoFps, setVideoFps] = useConversionSetting<string>("videoFps", "auto", oneOf(["auto", "24", "30", "50", "60"]))
  const [videoAudioCodec, setVideoAudioCodec] = useConversionSetting<string>("videoAudioCodec", "aac", oneOf(["aac", "mp3", "opus", "flac"]))
  const [videoAudioBitrate, setVideoAudioBitrate] = useConversionSetting<string>("videoAudioBitrate", "192", oneOf(["auto", "64", "96", "128", "160", "192", "256", "320"]))
  const [copyVideoStreams, setCopyVideoStreams] = useConversionSetting<boolean>("copyVideoStreams", false)
  const [progress, setProgress] = useState(0)
  const [elapsed, setElapsed] = useState(0)
  const startedAt = useRef<number | null>(null)
  const samples = useRef<ProgressSample[]>([])
  const [estimate, setEstimate] = useState<OutputEstimate | null>(null)
  const [isEstimating, setIsEstimating] = useState(false)
  const activeJobId = useRef<string | null>(null)
  const [conversionStatus, setConversionStatus] = useState<
    | { state: "idle" }
    | { state: "converting" }
    | { state: "completed"; path: string }
    | { state: "cancelled" }
    | { state: "error"; message: string }
  >({ state: "idle" })
  useEffect(() => {
    if (conversionStatus.state !== "converting") return
    const timer = window.setInterval(() => {
      if (startedAt.current !== null) setElapsed((performance.now() - startedAt.current) / 1000)
    }, 500)
    return () => window.clearInterval(timer)
  }, [conversionStatus.state])
  const handleAudioTrim = useCallback((trim: AudioTrim | null, valid: boolean) => {
    setAudioTrim(trim)
    setAudioTrimValid(valid)
    setConversionStatus({ state: "idle" })
  }, [])
  const mediaInfo = uploadedFile
    ? detectMediaFile(uploadedFile.name, uploadedFile.file?.type)
    : null
  const outputFormat = mediaInfo?.kind === "audio" ? audioFormat : mediaInfo?.kind === "video" ? videoFormat : imageFormat
  const setOutputFormat = mediaInfo?.kind === "audio" ? setAudioFormat : mediaInfo?.kind === "video" ? setVideoFormat : setImageFormat
  const availableFormats =
    mediaInfo?.kind === "audio"
      ? audioOutputFormats
      : mediaInfo?.kind === "video"
        ? videoOutputFormats
        : imageOutputFormats

  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) return
    let disposed = false
    let unlisten: (() => void) | undefined
    void listen<{ jobId: string; percent: number }>(
      "conversion-progress",
      (event) => {
        if (event.payload.jobId === activeJobId.current) {
          const percent = event.payload.percent
          if (!Number.isFinite(percent) || startedAt.current === null) return
          const seconds = (performance.now() - startedAt.current) / 1000
          const last = samples.current[samples.current.length - 1]
          if (!last || percent > last.percent) {
            samples.current = [...samples.current.filter((sample) => seconds - sample.seconds <= 20), { seconds, percent }]
          }
          setProgress((previous) => Math.max(previous, Math.min(99.5, Math.max(0, percent))))
          setElapsed(seconds)
        }
      },
    ).then((cleanup) => {
      if (disposed) cleanup()
      else unlisten = cleanup
    })
    return () => {
      disposed = true
      unlisten?.()
    }
  }, [])

  useEffect(() => {
    if (!uploadedFile?.path || !mediaInfo || mediaInfo.kind === "unknown") {
      setEstimate(null)
      return
    }
    let active = true
    setEstimate(null)
    setIsEstimating(true)
    const timeout = window.setTimeout(() => {
      setIsEstimating(true)
      void invoke<OutputEstimate | null>("estimate_output", {
        input: uploadedFile.path,
        mediaKind: mediaInfo.kind,
        format: outputFormat.toLowerCase(),
        sampleRateHz: audioSampleRate === "auto" ? null : Number(audioSampleRate),
        channels: audioChannels === "source" ? null : Number(audioChannels),
        audioBitrateKbps:
          mediaInfo.kind === "audio" &&
          !["WAV", "FLAC", "AIFF"].includes(outputFormat) &&
          audioBitrate !== "auto"
            ? Number(audioBitrate)
            : null,
        videoCodec: mediaInfo.kind === "video" ? videoCodec : null,
        crf: mediaInfo.kind === "video" ? videoCrf : null,
        height:
          mediaInfo.kind === "video" && videoHeight !== "auto"
            ? Number(videoHeight)
            : null,
        copyStreams: mediaInfo.kind === "video" && copyVideoStreams,
        trimStart: mediaInfo.kind === "audio" ? audioTrim?.start ?? null : null,
        trimEnd: mediaInfo.kind === "audio" ? audioTrim?.end ?? null : null,
        tempo: mediaInfo.kind === "audio" ? audioTempo : null,
      })
        .then((value) => {
          if (active) setEstimate(value)
        })
        .catch(() => {
          if (active) setEstimate(null)
        })
        .finally(() => {
          if (active) setIsEstimating(false)
        })
    }, 250)
    return () => {
      active = false
      window.clearTimeout(timeout)
    }
  }, [
    uploadedFile?.path,
    mediaInfo?.kind,
    outputFormat,
    audioBitrate,
    audioSampleRate,
    audioChannels,
    videoCodec,
    videoCrf,
    videoHeight,
    copyVideoStreams,
    audioTrim?.start,
    audioTrim?.end,
    audioTempo,
  ])

  async function convertMedia() {
    if (
      !uploadedFile?.path ||
      (mediaInfo?.kind === "audio" && !audioTrimValid) ||
      (mediaInfo?.kind !== "image" &&
        mediaInfo?.kind !== "audio" &&
        mediaInfo?.kind !== "video")
    ) return

    setConversionStatus({ state: "converting" })
    setProgress(0)
    setElapsed(0)
    startedAt.current = performance.now()
    samples.current = []

    try {
      const command =
        mediaInfo.kind === "image"
          ? "convert_image"
          : mediaInfo.kind === "audio"
            ? "convert_audio"
            : "convert_video"
      const jobId =
        mediaInfo.kind === "image" ? null : crypto.randomUUID()
      activeJobId.current = jobId
      const result = await invoke<{ path: string }>(command, {
        ...(jobId ? { jobId } : {}),
        input: uploadedFile.path,
        outputDirectory: outputDirectory || null,
        format: outputFormat.toLowerCase(),
        ...(mediaInfo.kind === "image" ? { quality, stripMetadata } : {}),
        ...(mediaInfo.kind === "audio"
          ? {
              bitrateKbps:
                ["WAV", "FLAC", "AIFF"].includes(outputFormat) ||
                audioBitrate === "auto"
                  ? null
                  : Number(audioBitrate),
              sampleRateHz:
                audioSampleRate === "auto" ? null : Number(audioSampleRate),
              channels:
                audioChannels === "source" ? null : Number(audioChannels),
              volumeDb: audioVolume === 0 ? null : audioVolume,
              tempo: audioTempo === 1 ? null : audioTempo,
              pitchSemitones: audioPitch === 0 ? null : audioPitch,
              normalizeLufs: normalizeAudio,
              targetLufs: normalizeAudio ? Number(audioTargetLufs) : null,
              trimStart: audioTrim?.start ?? null,
              trimEnd: audioTrim?.end ?? null,
              stripMetadata,
            }
          : {}),
        ...(mediaInfo.kind === "video"
          ? {
              videoCodec,
              crf: videoCrf,
              height: videoHeight === "auto" ? null : Number(videoHeight),
              fps: videoFps === "auto" ? null : Number(videoFps),
              audioCodec: videoAudioCodec,
              audioBitrateKbps:
                videoAudioCodec === "flac" || videoAudioBitrate === "auto"
                  ? null
                  : Number(videoAudioBitrate),
              stripMetadata,
              copyStreams: copyVideoStreams,
            }
          : {}),
      })
      setProgress(100)
      activeJobId.current = null
      setConversionStatus({ state: "completed", path: result.path })
      try {
        addConversionRecord({
          id: crypto.randomUUID(),
          inputName: uploadedFile.name,
          inputPath: uploadedFile.path,
          outputPath: result.path,
          format: outputFormat,
          completedAt: new Date().toISOString(),
        })
        setHistoryWarning("")
      } catch {
        setHistoryWarning("Файл сохранён, но записать конвертацию в историю не удалось.")
      }
    } catch (error) {
      activeJobId.current = null
      const wasCancelled =
        (typeof error === "string" &&
          error.toLowerCase().includes("cancelled")) ||
        (error && typeof error === "object" && "Cancelled" in error)
      if (wasCancelled) {
        setConversionStatus({ state: "cancelled" })
        return
      }
      const backendMessage =
        error && typeof error === "object"
          ? Object.values(error).find((value) => typeof value === "string")
          : undefined
      setConversionStatus({
        state: "error",
        message:
          typeof error === "string"
            ? error
            : typeof backendMessage === "string"
              ? backendMessage
              : "Не удалось конвертировать файл.",
      })
    }
  }

  async function cancelConversion() {
    if (!activeJobId.current) return
    try {
      await invoke("cancel_convert", { jobId: activeJobId.current })
    } catch {
      // Процесс мог завершиться между нажатием кнопки и командой отмены.
    }
  }

  return (
    <main className="min-h-0 min-w-0 flex-1 overflow-y-auto p-6">
      <div className="mx-auto flex w-full max-w-3xl flex-col gap-6">
        {historyWarning && <p role="alert" className="text-sm text-destructive">{historyWarning}</p>}
        <div>
          <h1 className="text-3xl font-semibold tracking-tight">Конвертация</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Добавьте файл, который хотите конвертировать.
          </p>
        </div>
        <MediaFileUploader
          disabled={conversionStatus.state === "converting"}
          onFileChange={(file) => {
            setUploadedFile(file)
            setAudioTrim(null)
            setAudioTrimValid(true)
            setConversionStatus({ state: "idle" })
          }}
        />

        {mediaInfo?.kind === "audio" && uploadedFile && (
          <AudioTrimmer
            key={uploadedFile.path ?? `${uploadedFile.name}-${uploadedFile.file?.lastModified}`}
            path={uploadedFile.path}
            file={uploadedFile.file}
            disabled={conversionStatus.state === "converting"}
            onChange={handleAudioTrim}
          />
        )}

        {(mediaInfo?.kind === "image" ||
          mediaInfo?.kind === "audio" ||
          mediaInfo?.kind === "video") && (
        <section className="space-y-5 rounded-xl border bg-card p-5 text-card-foreground shadow-xs">
          <fieldset disabled={conversionStatus.state === "converting"} className="min-w-0 space-y-5 disabled:opacity-60">
          <div className="space-y-2">
            <h2 className="text-sm font-medium">Формат результата</h2>
            <Tabs
              key={uploadedFile?.name}
              value={outputFormat}
              onValueChange={(value) => {
                setOutputFormat(value)
                setConversionStatus({ state: "idle" })
              }}
            >
              <TabsList className="h-auto flex-wrap">
                {availableFormats.map((format) => (
                  <TabsTrigger key={format} value={format} className="px-3 py-1">
                    {format}
                  </TabsTrigger>
                ))}
              </TabsList>
              {availableFormats.map((format) => (
                <TabsContent key={format} value={format} className="text-muted-foreground">
                  Настройки формата {format} появятся здесь позже.
                </TabsContent>
              ))}
            </Tabs>
          </div>

          {mediaInfo.kind === "image" &&
            (outputFormat === "JPG" || outputFormat === "AVIF") && (
            <div className="space-y-3">
              <div className="flex items-center justify-between gap-4">
                <label htmlFor="image-quality" className="text-sm font-medium">
                  Качество
                </label>
                <div className="flex items-center gap-1.5">
                  <Input
                    type="number"
                    min={0}
                    max={100}
                    value={quality}
                    aria-label="Качество изображения в процентах"
                    className="w-20 text-right tabular-nums"
                    onChange={(event) => {
                      const nextValue = Number(event.target.value)
                      setQuality(
                        Math.min(
                          100,
                          Math.max(0, Number.isNaN(nextValue) ? 0 : nextValue),
                        ),
                      )
                      setConversionStatus({ state: "idle" })
                    }}
                  />
                  <span className="text-sm text-muted-foreground">%</span>
                </div>
              </div>
              <input
                id="image-quality"
                type="range"
                min={0}
                max={100}
                step={1}
                value={quality}
                onChange={(event) => {
                  setQuality(Number(event.target.value))
                  setConversionStatus({ state: "idle" })
                }}
                className="h-2 w-full cursor-pointer appearance-none rounded-full bg-muted accent-primary"
              />
              <div className="flex justify-between text-xs text-muted-foreground">
                <span>Меньше размер</span>
                <span>Выше качество</span>
              </div>
            </div>
          )}

          {mediaInfo.kind === "image" && outputFormat === "WEBP" && (
            <p className="text-sm text-muted-foreground">
              WebP пока сохраняется без потери качества.
            </p>
          )}

          {mediaInfo.kind === "audio" && (
            <div className="grid gap-4 sm:grid-cols-2">
              <label className="space-y-2">
                <span className="block text-sm font-medium">Битрейт</span>
                <select
                  value={audioBitrate}
                  disabled={["WAV", "FLAC", "AIFF"].includes(outputFormat)}
                  onChange={(event) => {
                    setAudioBitrate(event.target.value)
                    setConversionStatus({ state: "idle" })
                  }}
                  className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50"
                >
                  <option value="auto">Автоматически</option>
                  {[96, 128, 192, 256, 320].map((bitrate) => (
                    <option key={bitrate} value={bitrate}>
                      {bitrate} кбит/с
                    </option>
                  ))}
                </select>
                <span className="block text-xs text-muted-foreground">
                  {["WAV", "FLAC", "AIFF"].includes(outputFormat)
                    ? "Для форматов без потерь битрейт не задаётся."
                    : "Выше битрейт — лучше качество и больше размер файла."}
                </span>
              </label>

              <label className="space-y-2">
                <span className="block text-sm font-medium">
                  Частота дискретизации
                </span>
                <select
                  value={audioSampleRate}
                  onChange={(event) => {
                    setAudioSampleRate(event.target.value)
                    setConversionStatus({ state: "idle" })
                  }}
                  className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
                >
                  <option value="auto">Исходная</option>
                  <option value="22050">22,05 кГц</option>
                  <option value="44100">44,1 кГц</option>
                  <option value="48000">48 кГц</option>
                  <option value="96000">96 кГц</option>
                </select>
                <span className="block text-xs text-muted-foreground">
                  Для музыки обычно достаточно 44,1 кГц, для видео — 48 кГц.
                </span>
              </label>

              <label className="space-y-2">
                <span className="block text-sm font-medium">Каналы</span>
                <select
                  value={audioChannels}
                  onChange={(event) => {
                    setAudioChannels(event.target.value)
                    setConversionStatus({ state: "idle" })
                  }}
                  className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
                >
                  <option value="source">Исходные</option>
                  <option value="1">Моно</option>
                  <option value="2">Стерео</option>
                </select>
                <span className="block text-xs text-muted-foreground">
                  Моно уменьшает размер и подходит для речи.
                </span>
              </label>

              <div className="space-y-3 sm:col-span-2">
                <div className="flex items-center justify-between gap-4">
                  <label htmlFor="audio-volume" className="text-sm font-medium">
                    Изменение громкости
                  </label>
                  <div className="flex items-center gap-1.5">
                    <Input
                      type="number"
                      min={-60}
                      max={30}
                      step={1}
                      value={audioVolume}
                      aria-label="Изменение громкости в децибелах"
                      className="w-20 text-right tabular-nums"
                      onChange={(event) => {
                        const value = Number(event.target.value)
                        setAudioVolume(Math.min(30, Math.max(-60, value || 0)))
                        setConversionStatus({ state: "idle" })
                      }}
                    />
                    <span className="text-sm text-muted-foreground">дБ</span>
                  </div>
                </div>
                <input
                  id="audio-volume"
                  type="range"
                  min={-24}
                  max={30}
                  step={1}
                  value={audioVolume}
                  onChange={(event) => {
                    setAudioVolume(Number(event.target.value))
                    setConversionStatus({ state: "idle" })
                  }}
                  className="h-2 w-full cursor-pointer appearance-none rounded-full bg-muted accent-primary"
                />
                <div className="flex justify-between text-xs text-muted-foreground">
                  <span>Тише</span>
                  <span>Без изменений: 0 дБ</span>
                  <span>Громче</span>
                </div>
              </div>

              <div className="space-y-3 sm:col-span-2">
                <div className="flex items-center justify-between gap-4">
                  <label htmlFor="audio-tempo" className="text-sm font-medium">
                    Темп
                  </label>
                  <div className="flex items-center gap-1.5">
                    <Input
                      type="number"
                      min={0.5}
                      max={2}
                      step={0.05}
                      value={audioTempo}
                      aria-label="Множитель темпа"
                      className="w-20 text-right tabular-nums"
                      onChange={(event) => {
                        const value = Number(event.target.value)
                        setAudioTempo(Math.min(2, Math.max(0.5, value || 1)))
                        setConversionStatus({ state: "idle" })
                      }}
                    />
                    <span className="text-sm text-muted-foreground">×</span>
                  </div>
                </div>
                <input
                  id="audio-tempo"
                  type="range"
                  min={0.5}
                  max={2}
                  step={0.05}
                  value={audioTempo}
                  onChange={(event) => {
                    setAudioTempo(Number(event.target.value))
                    setConversionStatus({ state: "idle" })
                  }}
                  className="h-2 w-full cursor-pointer appearance-none rounded-full bg-muted accent-primary"
                />
                <div className="flex justify-between text-xs text-muted-foreground">
                  <span>Медленнее: 0,5×</span>
                  <span>Исходный: 1×</span>
                  <span>Быстрее: 2×</span>
                </div>
                <p className="text-xs text-muted-foreground">
                  Меняет длительность записи, сохраняя исходную высоту тона.
                </p>
              </div>

              <div className="space-y-3 sm:col-span-2">
                <div className="flex items-center justify-between gap-4">
                  <label htmlFor="audio-pitch" className="text-sm font-medium">
                    Высота тона
                  </label>
                  <div className="flex items-center gap-1.5">
                    <Input
                      type="number"
                      min={-12}
                      max={12}
                      step={1}
                      value={audioPitch}
                      aria-label="Сдвиг высоты тона в полутонах"
                      className="w-20 text-right tabular-nums"
                      onChange={(event) => {
                        const value = Number(event.target.value)
                        setAudioPitch(Math.min(12, Math.max(-12, value || 0)))
                        setConversionStatus({ state: "idle" })
                      }}
                    />
                    <span className="text-sm text-muted-foreground">полут.</span>
                  </div>
                </div>
                <input
                  id="audio-pitch"
                  type="range"
                  min={-12}
                  max={12}
                  step={1}
                  value={audioPitch}
                  onChange={(event) => {
                    setAudioPitch(Number(event.target.value))
                    setConversionStatus({ state: "idle" })
                  }}
                  className="h-2 w-full cursor-pointer appearance-none rounded-full bg-muted accent-primary"
                />
                <div className="flex justify-between text-xs text-muted-foreground">
                  <span>Ниже: −12</span>
                  <span>Без изменений: 0</span>
                  <span>Выше: +12</span>
                </div>
                <p className="text-xs text-muted-foreground">
                  Меняет высоту звука, сохраняя длительность записи.
                </p>
              </div>

              <label className="flex cursor-pointer items-start gap-3 rounded-lg border p-3 sm:col-span-2">
                <input
                  type="checkbox"
                  checked={normalizeAudio}
                  onChange={(event) => {
                    setNormalizeAudio(event.target.checked)
                    setConversionStatus({ state: "idle" })
                  }}
                  className="mt-0.5 size-4 shrink-0 cursor-pointer accent-primary"
                />
                <span className="min-w-0 flex-1 space-y-2">
                  <span className="block text-sm font-medium">
                    Нормализовать громкость по LUFS
                  </span>
                  <span className="block text-sm text-muted-foreground">
                    Выравнивает воспринимаемую громкость и ограничивает пики.
                  </span>
                  {normalizeAudio && (
                    <select
                      value={audioTargetLufs}
                      onChange={(event) => {
                        setAudioTargetLufs(event.target.value)
                        setConversionStatus({ state: "idle" })
                      }}
                      className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
                    >
                      <option value="-23">−23 LUFS — вещание</option>
                      <option value="-16">−16 LUFS — подкаст</option>
                      <option value="-14">−14 LUFS — музыка и стриминг</option>
                    </select>
                  )}
                </span>
              </label>

              <label className="flex cursor-pointer items-start gap-3 rounded-lg border p-3 sm:col-span-2">
                <input
                  type="checkbox"
                  checked={stripMetadata}
                  onChange={(event) => {
                    setStripMetadata(event.target.checked)
                    setConversionStatus({ state: "idle" })
                  }}
                  className="mt-0.5 size-4 shrink-0 cursor-pointer accent-primary"
                />
                <span className="space-y-0.5">
                  <span className="block text-sm font-medium">Удалить метаданные</span>
                  <span className="block text-sm text-muted-foreground">
                    Не переносить исполнителя, альбом, обложку и остальные теги.
                  </span>
                </span>
              </label>
            </div>
          )}

          {mediaInfo.kind === "video" && (
            <div className="space-y-5">
              <label className="flex cursor-pointer items-start gap-3 rounded-lg border p-3">
                <input
                  type="checkbox"
                  checked={copyVideoStreams}
                  onChange={(event) => {
                    setCopyVideoStreams(event.target.checked)
                    setConversionStatus({ state: "idle" })
                  }}
                  className="mt-0.5 size-4 shrink-0 cursor-pointer accent-primary"
                />
                <span className="space-y-0.5">
                  <span className="block text-sm font-medium">
                    Копировать потоки без перекодирования
                  </span>
                  <span className="block text-sm text-muted-foreground">
                    Быстро и без потери качества. Работает только при совместимости исходных потоков с выбранным контейнером.
                  </span>
                </span>
              </label>

              <fieldset
                disabled={copyVideoStreams}
                className="grid gap-4 disabled:opacity-50 sm:grid-cols-2"
              >
                <label className="space-y-2">
                  <span className="block text-sm font-medium">Видеокодек</span>
                  <select
                    value={videoCodec}
                    onChange={(event) => {
                      setVideoCodec(event.target.value)
                      setConversionStatus({ state: "idle" })
                    }}
                    className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
                  >
                    <option value="h264">H.264</option>
                    <option value="h265">H.265 / HEVC</option>
                    <option value="vp9">VP9</option>
                    <option value="av1">AV1</option>
                  </select>
                </label>

                <label className="space-y-2">
                  <span className="block text-sm font-medium">Разрешение</span>
                  <select
                    value={videoHeight}
                    onChange={(event) => {
                      setVideoHeight(event.target.value)
                      setConversionStatus({ state: "idle" })
                    }}
                    className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
                  >
                    <option value="auto">Исходное</option>
                    <option value="2160">4K (2160p)</option>
                    <option value="1440">1440p</option>
                    <option value="1080">1080p</option>
                    <option value="720">720p</option>
                    <option value="480">480p</option>
                  </select>
                </label>

                <label className="space-y-2">
                  <span className="block text-sm font-medium">Частота кадров</span>
                  <select
                    value={videoFps}
                    onChange={(event) => {
                      setVideoFps(event.target.value)
                      setConversionStatus({ state: "idle" })
                    }}
                    className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
                  >
                    <option value="auto">Исходная</option>
                    {[24, 30, 50, 60].map((fps) => (
                      <option key={fps} value={fps}>{fps} FPS</option>
                    ))}
                  </select>
                </label>

                <label className="space-y-2">
                  <span className="block text-sm font-medium">Аудиокодек</span>
                  <select
                    value={videoAudioCodec}
                    onChange={(event) => {
                      setVideoAudioCodec(event.target.value)
                      setConversionStatus({ state: "idle" })
                    }}
                    className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
                  >
                    <option value="aac">AAC</option>
                    <option value="mp3">MP3</option>
                    <option value="opus">Opus</option>
                    <option value="flac">FLAC</option>
                  </select>
                </label>

                <label className="space-y-2 sm:col-span-2">
                  <span className="block text-sm font-medium">
                    Битрейт аудио
                  </span>
                  <select
                    value={videoAudioBitrate}
                    disabled={videoAudioCodec === "flac"}
                    onChange={(event) => {
                      setVideoAudioBitrate(event.target.value)
                      setConversionStatus({ state: "idle" })
                    }}
                    className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:cursor-not-allowed"
                  >
                    <option value="auto">Автоматически</option>
                    {[96, 128, 192, 256, 320].map((bitrate) => (
                      <option key={bitrate} value={bitrate}>
                        {bitrate} кбит/с
                      </option>
                    ))}
                  </select>
                </label>

                <div className="space-y-3 sm:col-span-2">
                  <div className="flex items-center justify-between gap-4">
                    <label htmlFor="video-crf" className="text-sm font-medium">
                      Качество CRF
                    </label>
                    <span className="text-sm tabular-nums">{videoCrf}</span>
                  </div>
                  <input
                    id="video-crf"
                    type="range"
                    min={0}
                    max={51}
                    value={videoCrf}
                    onChange={(event) => {
                      setVideoCrf(Number(event.target.value))
                      setConversionStatus({ state: "idle" })
                    }}
                    className="h-2 w-full cursor-pointer appearance-none rounded-full bg-muted accent-primary"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>Лучше качество</span>
                    <span>Меньше размер</span>
                  </div>
                </div>
              </fieldset>

              <label className="flex cursor-pointer items-start gap-3 rounded-lg border p-3">
                <input
                  type="checkbox"
                  checked={stripMetadata}
                  onChange={(event) => {
                    setStripMetadata(event.target.checked)
                    setConversionStatus({ state: "idle" })
                  }}
                  className="mt-0.5 size-4 shrink-0 cursor-pointer accent-primary"
                />
                <span className="space-y-0.5">
                  <span className="block text-sm font-medium">Удалить метаданные</span>
                  <span className="block text-sm text-muted-foreground">
                    Не переносить служебные теги исходного видео.
                  </span>
                </span>
              </label>
            </div>
          )}

          {mediaInfo.kind === "image" && (
          <label className="flex cursor-pointer items-start gap-3 rounded-lg border p-3">
            <input
              type="checkbox"
              checked={stripMetadata}
              onChange={(event) => {
                setStripMetadata(event.target.checked)
                setConversionStatus({ state: "idle" })
              }}
              className="mt-0.5 size-4 shrink-0 cursor-pointer accent-primary"
            />
            <span className="space-y-0.5">
              <span className="block text-sm font-medium">Удалить метаданные</span>
              <span className="block text-sm text-muted-foreground">
                Удалить EXIF и цветовой профиль ICC. Ориентация изображения сохранится визуально.
              </span>
            </span>
          </label>
          )}

          <OutputDirectoryPicker
            value={outputDirectory}
            onChange={setOutputDirectory}
            description="Можно изменить папку только для этой конвертации. Выбор не сохранится в настройках."
          />
          </fieldset>

          <div className="rounded-lg border bg-muted/30 p-3">
            <div className="flex items-center justify-between gap-4 text-sm">
              <span className="font-medium">Оценка размера результата</span>
              <span className="tabular-nums text-muted-foreground">
                {isEstimating
                  ? "Расчёт…"
                  : estimate
                    ? `≈ ${formatBytes(estimate.estimatedBytes)}`
                    : "Недоступна без пробного кодирования"}
              </span>
            </div>
            {estimate && (
              <p className="mt-1 text-xs text-muted-foreground">
                Ожидаемый диапазон: {formatBytes(estimate.lowerBytes)}–{formatBytes(estimate.upperBytes)}. Для видео с CRF размер приблизительный.
              </p>
            )}
          </div>

          <Progress
            value={conversionStatus.state === "converting" && (mediaInfo.kind === "image" || progress === 0) ? null : progress}
            className="[&_[data-slot=progress-track]]:h-3"
          >
            <ProgressLabel>
              {conversionStatus.state === "converting"
                ? mediaInfo.kind === "image" ? "Обрабатываем изображение…" : progress >= 99.5 ? "Завершаем сохранение…" : progress > 0 ? "Конвертируем…" : "Подготавливаем файл…"
                : conversionStatus.state === "completed" ? "Готово" : conversionStatus.state === "cancelled" ? "Отменено" : conversionStatus.state === "error" ? "Ошибка" : "Ожидание запуска"}
            </ProgressLabel>
            {!(conversionStatus.state === "converting" && (mediaInfo.kind === "image" || progress === 0)) && <ProgressValue />}
          </Progress>
          {conversionStatus.state === "converting" && (
            <div className="flex flex-wrap gap-x-6 gap-y-2 text-sm text-muted-foreground tabular-nums">
              <span>Прошло: {formatElapsed(elapsed)}</span>
              {(() => {
                const estimate = estimateProgress(samples.current, elapsed)
                return <>
                  {mediaInfo.kind !== "image" && <span>Осталось: {progress >= 99.5 ? "завершаем сохранение…" : estimate ? `≈ ${formatElapsed(Math.ceil(estimate.remainingSeconds))}` : "оцениваем…"}</span>}
                  {estimate && <span>Скорость: {estimate.percentPerMinute.toFixed(1)} %/мин</span>}
                </>
              })()}
            </div>
          )}

          {!uploadedFile?.path && (
            <p className="text-sm text-destructive">
              Для конвертации выберите файл через приложение Tauri или перетащите его из проводника.
            </p>
          )}

          {conversionStatus.state === "completed" && (
            <div className="flex gap-2 text-sm text-foreground">
              <CircleCheckIcon className="size-5 shrink-0 text-emerald-600" />
              <p className="min-w-0 break-all">Готово: {conversionStatus.path}</p>
            </div>
          )}

          {conversionStatus.state === "error" && (
            <div className="flex gap-2 text-sm text-destructive">
              <CircleAlertIcon className="size-5 shrink-0" />
              <p>{conversionStatus.message}</p>
            </div>
          )}

          {conversionStatus.state === "cancelled" && (
            <div className="flex gap-2 text-sm text-muted-foreground">
              <CircleAlertIcon className="size-5 shrink-0" />
              <p>Конвертация отменена. Незавершённый файл удалён.</p>
            </div>
          )}

          <div className="flex justify-end gap-2">
            {conversionStatus.state === "converting" && activeJobId.current && (
              <Button
                type="button"
                size="lg"
                variant="outline"
                onClick={() => void cancelConversion()}
              >
                Отменить
              </Button>
            )}
            <Button
              type="button"
              size="lg"
              disabled={!uploadedFile?.path || conversionStatus.state === "converting" || (mediaInfo.kind === "audio" && !audioTrimValid)}
              onClick={() => void convertMedia()}
            >
              {conversionStatus.state === "converting" && (
                <LoaderCircleIcon className="animate-spin" data-icon="inline-start" />
              )}
              {conversionStatus.state === "converting"
                ? "Конвертация…"
                : "Конвертировать"}
            </Button>
          </div>
        </section>
        )}

        {mediaInfo?.kind === "unknown" && (
          <section className="flex gap-3 rounded-xl border bg-card p-5 text-card-foreground shadow-xs">
            <CircleAlertIcon className="mt-0.5 size-5 shrink-0 text-muted-foreground" />
            <div>
              <h2 className="font-medium">Не удалось определить тип файла</h2>
              <p className="mt-1 text-sm text-muted-foreground">
                Выберите изображение, аудио или видео в известном формате.
              </p>
            </div>
          </section>
        )}
      </div>
    </main>
  )
}
