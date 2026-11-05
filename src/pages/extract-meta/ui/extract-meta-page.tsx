import { useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import {
  CalendarDaysIcon,
  CameraIcon,
  CpuIcon,
  Grid2X2Icon,
  ImageIcon,
  Layers3Icon,
  LoaderCircleIcon,
  FilmIcon,
  MusicIcon,
  RulerIcon,
  TagsIcon,
} from "lucide-react"

import {
  MediaFileUploader,
  type UploadedFile,
} from "@/features/upload-media-file"

type GerberMetadata = {
  cadProgram: string | null
  creationDate: string | null
  layerType: string | null
  side: string | null
  units: string | null
  coordinateFormat: {
    xIntegerDigits: number
    xDecimalDigits: number
    yIntegerDigits: number
    yDecimalDigits: number
    notation: string | null
    zeroSuppression: string | null
  } | null
  statistics: {
    lineCount: number
    arcCount: number
    flashCount: number
    regionCount: number
    apertureCount: number
    apertureMacroCount: number
    usedDCodes: number[]
  }
  apertures: Array<{
    dCode: number
    template: string
    shape: string
    modifiers: string[]
    hasHole: boolean | null
    usageCount: number
  }>
  dCodeValidation: {
    undefinedUsed: number[]
    unusedDefined: number[]
    duplicateDefinitions: number[]
  }
  structureValidation: {
    isValid: boolean
    issues: Array<{
      code: string
      severity: "error" | "warning"
      message: string
      command: string | null
    }>
  }
}

type ImageMetadata = {
  format: string
  mimeType: string
  fileSize: number
  width: number
  height: number
  megapixels: number
  aspectRatio: string
  colorType: string
  bitsPerPixel: number
  channelCount: number
  hasAlpha: boolean
  exif: Array<{ key: string; value: string }>
}

type MediaMetadata = {
  kind: "audio" | "video"
  format: string
  fileSize: number
  durationSeconds: number | null
  bitrate: number | null
  video: {
    codec: string
    width: number | null
    height: number | null
    frameRate: number | null
  } | null
  audio: {
    codec: string
    sampleRateHz: number | null
    channels: number | null
    channelLayout: string | null
  } | null
  tags: Array<{ key: string; value: string }>
}

type ExtractionState =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "success"; type: "gerber"; metadata: GerberMetadata }
  | { kind: "success"; type: "image"; metadata: ImageMetadata }
  | { kind: "success"; type: "media"; metadata: MediaMetadata }
  | { kind: "error"; message: string }

const gerberExtensions = [
  "gbr",
  "ger",
  "gtl",
  "gbl",
  "gts",
  "gbs",
  "gto",
  "gbo",
  "gtp",
  "gbp",
  "gm1",
  "gko",
]

const imageExtensions = ["jpg", "jpeg", "png", "webp", "gif", "bmp", "tif", "tiff", "ico", "avif"]
const audioExtensions = ["mp3", "wav", "flac", "aac", "ogg", "oga", "opus", "m4a", "wma", "aif", "aiff", "alac", "amr"]
const videoExtensions = ["mp4", "mov", "mkv", "webm", "avi", "flv", "wmv", "m4v", "mpg", "mpeg", "3gp", "ts", "mts", "m2ts", "ogv"]
const mediaExtensions = [...audioExtensions, ...videoExtensions]

function errorMessage(error: unknown) {
  if (typeof error === "string") return error
  if (error && typeof error === "object") {
    const values = Object.values(error)
    if (typeof values[0] === "string") return values[0]
  }
  return "Не удалось прочитать метаданные файла"
}

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} Б`
  const units = ["КБ", "МБ", "ГБ"]
  let value = bytes / 1024
  let unit = units[0]
  for (let index = 1; value >= 1024 && index < units.length; index += 1) {
    value /= 1024
    unit = units[index]
  }
  return `${value.toFixed(value >= 100 ? 0 : 1)} ${unit}`
}

function formatDuration(seconds: number | null) {
  if (seconds === null || !Number.isFinite(seconds)) return "Не определена"
  const rounded = Math.max(0, Math.round(seconds))
  const hours = Math.floor(rounded / 3600)
  const minutes = Math.floor((rounded % 3600) / 60)
  const remainder = rounded % 60
  return hours > 0
    ? `${hours}:${String(minutes).padStart(2, "0")}:${String(remainder).padStart(2, "0")}`
    : `${minutes}:${String(remainder).padStart(2, "0")}`
}

function formatBitrate(bitrate: number | null) {
  if (bitrate === null) return "Не определён"
  return bitrate >= 1_000_000
    ? `${(bitrate / 1_000_000).toFixed(1)} Мбит/с`
    : `${Math.round(bitrate / 1000)} Кбит/с`
}

const mediaTagLabels: Record<string, string> = {
  title: "Название",
  artist: "Исполнитель",
  albumArtist: "Исполнитель альбома",
  album: "Альбом",
  genre: "Жанр",
  date: "Дата",
}

const exifLabels: Record<string, string> = {
  cameraMake: "Производитель камеры",
  cameraModel: "Модель камеры",
  lensModel: "Объектив",
  dateTaken: "Дата съёмки",
  dateDigitized: "Дата оцифровки",
  software: "Программа обработки",
  artist: "Автор",
  copyright: "Авторские права",
  exposureTime: "Выдержка",
  aperture: "Диафрагма",
  iso: "ISO",
  focalLength: "Фокусное расстояние",
  focalLength35mm: "Эквивалент 35 мм",
  exposureBias: "Экспокоррекция",
  flash: "Вспышка",
  orientation: "Ориентация",
  whiteBalance: "Баланс белого",
  gpsPosition: "GPS-координаты",
}

function layerTypeLabel(value: string | null) {
  if (!value) return "Не указан"
  const labels: Record<string, string> = {
    Copper: "Медь",
    Soldermask: "Паяльная маска",
    Legend: "Шелкография",
    Paste: "Паяльная паста",
    Profile: "Контур платы",
  }
  return labels[value] ? `${labels[value]} (${value})` : value
}

function sideLabel(value: string | null) {
  return value === "top"
    ? "Верхняя"
    : value === "bottom"
      ? "Нижняя"
      : value === "inner"
        ? "Внутренний слой"
        : "Не указана"
}

function coordinateFormatLabel(metadata: GerberMetadata) {
  const format = metadata.coordinateFormat
  if (!format) return "Не указан"
  const details = [
    `X ${format.xIntegerDigits}.${format.xDecimalDigits}`,
    `Y ${format.yIntegerDigits}.${format.yDecimalDigits}`,
  ]
  if (format.notation === "absolute") details.push("абсолютные координаты")
  if (format.notation === "incremental") details.push("относительные координаты")
  if (format.zeroSuppression === "leading") details.push("ведущие нули опущены")
  if (format.zeroSuppression === "trailing") details.push("конечные нули опущены")
  return details.join(" · ")
}

function apertureShapeLabel(shape: string, template: string) {
  const labels: Record<string, string> = {
    circle: "Круг",
    rectangle: "Прямоугольник",
    obround: "Овал",
    polygon: "Полигон",
  }
  return labels[shape] ?? `Макрос ${template}`
}

function dCodesLabel(codes: number[]) {
  return codes.map((code) => `D${code}`).join(", ")
}

export function ExtractMetaPage() {
  const [state, setState] = useState<ExtractionState>({ kind: "idle" })

  async function extract(file: UploadedFile | null) {
    if (!file) {
      setState({ kind: "idle" })
      return
    }
    if (!file.path) {
      setState({
        kind: "error",
        message: "Извлечение доступно в desktop-приложении. Выберите файл через Tauri или перетащите его из проводника.",
      })
      return
    }

    setState({ kind: "loading" })
    try {
      const extension = file.name.split(".").pop()?.toLowerCase() ?? ""
      if (imageExtensions.includes(extension)) {
        const metadata = await invoke<ImageMetadata>("extract_image_metadata", {
          path: file.path,
        })
        setState({ kind: "success", type: "image", metadata })
      } else if (mediaExtensions.includes(extension)) {
        const metadata = await invoke<MediaMetadata>("probe_file", {
          path: file.path,
        })
        setState({ kind: "success", type: "media", metadata })
      } else if (gerberExtensions.includes(extension)) {
        const metadata = await invoke<GerberMetadata>(
          "extract_gerber_metadata",
          { path: file.path },
        )
        setState({ kind: "success", type: "gerber", metadata })
      } else {
        setState({ kind: "error", message: "Этот тип файла пока не поддерживается." })
      }
    } catch (error) {
      setState({ kind: "error", message: errorMessage(error) })
    }
  }

  return (
    <main className="min-h-0 min-w-0 flex-1 overflow-y-auto p-6">
      <div className="mx-auto flex max-w-4xl flex-col gap-6">
        <div>
          <h1 className="text-3xl font-semibold tracking-tight">Извлечение Meta</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Посмотрите свойства изображений, аудио и видео или параметры и статистику Gerber.
          </p>
        </div>

        <MediaFileUploader
          disabled={state.kind === "loading"}
          accept={[...imageExtensions, ...mediaExtensions, ...gerberExtensions].map((extension) => `.${extension}`).join(",")}
          dialogExtensions={[...imageExtensions, ...mediaExtensions, ...gerberExtensions]}
          dialogTitle="Выберите медиа- или Gerber-файл"
          description="Перетащите изображение, аудио, видео или Gerber-файл, чтобы посмотреть основные метаданные."
          fileTypeLabel="Медиа или Gerber"
          onFileChange={(file) => void extract(file)}
        />

        {state.kind === "loading" && (
          <section className="flex items-center gap-3 rounded-xl border bg-card p-5 text-sm text-muted-foreground">
            <LoaderCircleIcon className="size-5 animate-spin" />
            Читаем атрибуты файла…
          </section>
        )}

        {state.kind === "error" && (
          <section className="rounded-xl border border-destructive/40 bg-destructive/5 p-5 text-sm text-destructive">
            {state.message}
          </section>
        )}

        {state.kind === "success" && state.type === "image" && (
          <>
            <section className="rounded-xl border bg-card p-5 text-card-foreground">
              <div className="flex items-start gap-3">
                <ImageIcon className="mt-0.5 size-5 text-muted-foreground" />
                <div>
                  <h2 className="font-medium">Свойства изображения</h2>
                  <p className="mt-1 text-sm text-muted-foreground">
                    Характеристики определены по содержимому файла.
                  </p>
                </div>
              </div>
              <dl className="mt-5 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                {[
                  ["Формат", `${state.metadata.format} · ${state.metadata.mimeType}`],
                  ["Размер файла", formatBytes(state.metadata.fileSize)],
                  ["Разрешение", `${state.metadata.width} × ${state.metadata.height} px`],
                  ["Размер изображения", `${state.metadata.megapixels.toFixed(2)} Мп`],
                  ["Соотношение сторон", state.metadata.aspectRatio],
                  ["Цветовая модель", state.metadata.colorType],
                  ["Глубина", `${state.metadata.bitsPerPixel} бит/пиксель`],
                  ["Каналы", String(state.metadata.channelCount)],
                  ["Прозрачность", state.metadata.hasAlpha ? "Есть" : "Нет"],
                ].map(([label, value]) => (
                  <div key={label} className="rounded-lg border bg-muted/30 p-4">
                    <dt className="text-sm text-muted-foreground">{label}</dt>
                    <dd className="mt-1 break-words font-medium">{value}</dd>
                  </div>
                ))}
              </dl>
            </section>

            <section className="overflow-hidden rounded-xl border bg-card text-card-foreground">
              <div className="flex items-start gap-3 border-b px-5 py-4">
                <CameraIcon className="mt-0.5 size-5 text-muted-foreground" />
                <div>
                  <h2 className="font-medium">EXIF-метаданные</h2>
                  <p className="mt-1 text-sm text-muted-foreground">
                    Данные камеры и съёмки показываются только при наличии в файле.
                  </p>
                </div>
              </div>
              {state.metadata.exif.length ? (
                <dl className="divide-y">
                  {state.metadata.exif.map((field) => (
                    <div key={field.key} className="grid gap-1 px-5 py-3 sm:grid-cols-[14rem_1fr] sm:gap-4">
                      <dt className="text-sm text-muted-foreground">
                        {exifLabels[field.key] ?? field.key}
                      </dt>
                      <dd className="break-words text-sm font-medium">{field.value}</dd>
                    </div>
                  ))}
                </dl>
              ) : (
                <p className="px-5 py-6 text-sm text-muted-foreground">
                  EXIF-метаданные не найдены. Такое нормально для PNG, скриншотов и изображений после очистки метаданных.
                </p>
              )}
            </section>
          </>
        )}

        {state.kind === "success" && state.type === "media" && (
          <>
            <section className="rounded-xl border bg-card p-5 text-card-foreground">
              <div className="flex items-start gap-3">
                {state.metadata.kind === "video" ? (
                  <FilmIcon className="mt-0.5 size-5 text-muted-foreground" />
                ) : (
                  <MusicIcon className="mt-0.5 size-5 text-muted-foreground" />
                )}
                <div>
                  <h2 className="font-medium">
                    {state.metadata.kind === "video" ? "Свойства видео" : "Свойства аудио"}
                  </h2>
                  <p className="mt-1 text-sm text-muted-foreground">
                    Основные параметры прочитаны с помощью FFprobe.
                  </p>
                </div>
              </div>
              <dl className="mt-5 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                {[
                  ["Формат", state.metadata.format],
                  ["Размер файла", formatBytes(state.metadata.fileSize)],
                  ["Длительность", formatDuration(state.metadata.durationSeconds)],
                  ["Общий битрейт", formatBitrate(state.metadata.bitrate)],
                  ...(state.metadata.video
                    ? [
                        ["Видеокодек", state.metadata.video.codec],
                        [
                          "Разрешение",
                          state.metadata.video.width && state.metadata.video.height
                            ? `${state.metadata.video.width} × ${state.metadata.video.height} px`
                            : "Не определено",
                        ],
                        [
                          "Частота кадров",
                          state.metadata.video.frameRate
                            ? `${state.metadata.video.frameRate.toFixed(2)} FPS`
                            : "Не определена",
                        ],
                      ]
                    : []),
                  ...(state.metadata.audio
                    ? [
                        ["Аудиокодек", state.metadata.audio.codec],
                        [
                          "Частота дискретизации",
                          state.metadata.audio.sampleRateHz
                            ? `${(state.metadata.audio.sampleRateHz / 1000).toFixed(1)} кГц`
                            : "Не определена",
                        ],
                        [
                          "Каналы",
                          state.metadata.audio.channels
                            ? `${state.metadata.audio.channels}${state.metadata.audio.channelLayout ? ` · ${state.metadata.audio.channelLayout}` : ""}`
                            : "Не определены",
                        ],
                      ]
                    : []),
                ].map(([label, value]) => (
                  <div key={label} className="rounded-lg border bg-muted/30 p-4">
                    <dt className="text-sm text-muted-foreground">{label}</dt>
                    <dd className="mt-1 break-words font-medium">{value}</dd>
                  </div>
                ))}
              </dl>
            </section>

            {state.metadata.tags.length > 0 && (
              <section className="overflow-hidden rounded-xl border bg-card text-card-foreground">
                <div className="flex items-start gap-3 border-b px-5 py-4">
                  <TagsIcon className="mt-0.5 size-5 text-muted-foreground" />
                  <div>
                    <h2 className="font-medium">Встроенные теги</h2>
                    <p className="mt-1 text-sm text-muted-foreground">
                      Показываются только заполненные основные поля.
                    </p>
                  </div>
                </div>
                <dl className="divide-y">
                  {state.metadata.tags.map((field) => (
                    <div key={field.key} className="grid gap-1 px-5 py-3 sm:grid-cols-[14rem_1fr] sm:gap-4">
                      <dt className="text-sm text-muted-foreground">
                        {mediaTagLabels[field.key] ?? field.key}
                      </dt>
                      <dd className="break-words text-sm font-medium">{field.value}</dd>
                    </div>
                  ))}
                </dl>
              </section>
            )}
          </>
        )}

        {state.kind === "success" && state.type === "gerber" && (
          <>
            <section className="rounded-xl border bg-card text-card-foreground">
              <div className="border-b px-5 py-4">
                <h2 className="font-medium">Метаданные Gerber</h2>
                <p className="mt-1 text-sm text-muted-foreground">
                  Пустые значения означают, что автор файла не записал этот атрибут.
                </p>
              </div>
              <dl className="divide-y">
              <div className="flex items-start gap-4 p-5">
                <CpuIcon className="mt-0.5 size-5 shrink-0 text-muted-foreground" />
                <div className="min-w-0">
                  <dt className="text-sm text-muted-foreground">CAD-программа</dt>
                  <dd className="mt-1 break-words font-medium">
                    {state.metadata.cadProgram ?? "Не указана"}
                  </dd>
                </div>
              </div>
              <div className="flex items-start gap-4 p-5">
                <CalendarDaysIcon className="mt-0.5 size-5 shrink-0 text-muted-foreground" />
                <div className="min-w-0">
                  <dt className="text-sm text-muted-foreground">Дата создания</dt>
                  <dd className="mt-1 break-words font-medium">
                    {state.metadata.creationDate ?? "Не указана"}
                  </dd>
                </div>
              </div>
              <div className="flex items-start gap-4 p-5">
                <Layers3Icon className="mt-0.5 size-5 shrink-0 text-muted-foreground" />
                <div className="min-w-0">
                  <dt className="text-sm text-muted-foreground">Тип слоя</dt>
                  <dd className="mt-1 break-words font-medium">
                    {layerTypeLabel(state.metadata.layerType)}
                  </dd>
                </div>
              </div>
              <div className="flex items-start gap-4 p-5">
                <Grid2X2Icon className="mt-0.5 size-5 shrink-0 text-muted-foreground" />
                <div className="min-w-0">
                  <dt className="text-sm text-muted-foreground">Сторона</dt>
                  <dd className="mt-1 break-words font-medium">
                    {sideLabel(state.metadata.side)}
                  </dd>
                </div>
              </div>
              <div className="flex items-start gap-4 p-5">
                <RulerIcon className="mt-0.5 size-5 shrink-0 text-muted-foreground" />
                <div className="min-w-0">
                  <dt className="text-sm text-muted-foreground">Единицы измерения</dt>
                  <dd className="mt-1 break-words font-medium">
                    {state.metadata.units === "mm"
                      ? "Миллиметры (мм)"
                      : state.metadata.units === "inch"
                        ? "Дюймы"
                        : "Не указаны"}
                  </dd>
                </div>
              </div>
              <div className="flex items-start gap-4 p-5">
                <Grid2X2Icon className="mt-0.5 size-5 shrink-0 text-muted-foreground" />
                <div className="min-w-0">
                  <dt className="text-sm text-muted-foreground">Формат координат</dt>
                  <dd className="mt-1 break-words font-medium">
                    {coordinateFormatLabel(state.metadata)}
                  </dd>
                </div>
              </div>
              </dl>
            </section>

            <section className="rounded-xl border bg-card p-5 text-card-foreground">
              <div>
                <h2 className="font-medium">Статистика файла</h2>
                <p className="mt-1 text-sm text-muted-foreground">
                  Объекты подсчитаны по операциям и определениям Gerber.
                </p>
              </div>
              <dl className="mt-5 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                {[
                  ["Линии", state.metadata.statistics.lineCount],
                  ["Дуги", state.metadata.statistics.arcCount],
                  ["Вспышки", state.metadata.statistics.flashCount],
                  ["Области", state.metadata.statistics.regionCount],
                  ["Апертуры", state.metadata.statistics.apertureCount],
                  ["Макросы апертур", state.metadata.statistics.apertureMacroCount],
                ].map(([label, value]) => (
                  <div key={label} className="rounded-lg border bg-muted/30 p-4">
                    <dt className="text-sm text-muted-foreground">{label}</dt>
                    <dd className="mt-1 text-2xl font-semibold tabular-nums">{value}</dd>
                  </div>
                ))}
              </dl>
              <div className="mt-5 border-t pt-5">
                <h3 className="text-sm text-muted-foreground">Используемые D-коды апертур</h3>
                <div className="mt-2 flex flex-wrap gap-2">
                  {state.metadata.statistics.usedDCodes.length ? (
                    state.metadata.statistics.usedDCodes.map((code) => (
                      <span
                        key={code}
                        className="rounded-md border bg-muted px-2.5 py-1 font-mono text-sm"
                      >
                        D{code}
                      </span>
                    ))
                  ) : (
                    <span className="text-sm font-medium">Не найдены</span>
                  )}
                </div>
              </div>
            </section>

            <section className="overflow-hidden rounded-xl border bg-card text-card-foreground">
              <div className="border-b px-5 py-4">
                <h2 className="font-medium">Проверка структуры Gerber</h2>
                <p className="mt-1 text-sm text-muted-foreground">
                  Обязательные команды, области, апертуры и совместимость команд со стандартом.
                </p>
              </div>

              {state.metadata.structureValidation.issues.length === 0 ? (
                <div className="bg-emerald-500/5 px-5 py-4 text-sm text-emerald-700 dark:text-emerald-400">
                  Ошибок и предупреждений не найдено.
                </div>
              ) : (
                <div className="divide-y">
                  {state.metadata.structureValidation.issues.map((issue, index) => (
                    <div
                      key={`${issue.code}-${issue.command ?? index}`}
                      className="flex flex-col gap-2 px-5 py-4 sm:flex-row sm:items-start"
                    >
                      <span
                        className={
                          issue.severity === "error"
                            ? "w-fit rounded-md bg-destructive/10 px-2 py-1 text-xs font-medium text-destructive"
                            : "w-fit rounded-md bg-amber-500/10 px-2 py-1 text-xs font-medium text-amber-800 dark:text-amber-300"
                        }
                      >
                        {issue.severity === "error" ? "Ошибка" : "Предупреждение"}
                      </span>
                      <div className="min-w-0 flex-1">
                        <p className="text-sm">{issue.message}</p>
                        {issue.command && (
                          <code className="mt-1 block break-all text-xs text-muted-foreground">
                            {issue.command}
                          </code>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </section>

            <section className="overflow-hidden rounded-xl border bg-card text-card-foreground">
              <div className="border-b px-5 py-4">
                <h2 className="font-medium">Апертуры и проверка D-кодов</h2>
                <p className="mt-1 text-sm text-muted-foreground">
                  Использование считается по операциям рисования и вспышкам.
                </p>
              </div>

              {state.metadata.dCodeValidation.undefinedUsed.length === 0 &&
              state.metadata.dCodeValidation.duplicateDefinitions.length === 0 ? (
                <div className="border-b bg-emerald-500/5 px-5 py-4 text-sm text-emerald-700 dark:text-emerald-400">
                  Используемые D-коды определены, повторных определений нет.
                </div>
              ) : (
                <div className="space-y-1 border-b bg-amber-500/10 px-5 py-4 text-sm text-amber-800 dark:text-amber-300">
                  {state.metadata.dCodeValidation.undefinedUsed.length > 0 && (
                    <p>
                      Используются без определения: {dCodesLabel(state.metadata.dCodeValidation.undefinedUsed)}
                    </p>
                  )}
                  {state.metadata.dCodeValidation.duplicateDefinitions.length > 0 && (
                    <p>
                      Повторные определения: {dCodesLabel(state.metadata.dCodeValidation.duplicateDefinitions)}
                    </p>
                  )}
                </div>
              )}

              {state.metadata.dCodeValidation.unusedDefined.length > 0 && (
                <div className="border-b px-5 py-3 text-sm text-muted-foreground">
                  Определены, но не используются: {dCodesLabel(state.metadata.dCodeValidation.unusedDefined)}
                </div>
              )}

              {state.metadata.apertures.length ? (
                <div className="overflow-x-auto">
                  <table className="w-full min-w-2xl text-left text-sm">
                    <thead className="bg-muted/50 text-muted-foreground">
                      <tr>
                        <th className="px-5 py-3 font-medium">D-код</th>
                        <th className="px-5 py-3 font-medium">Форма</th>
                        <th className="px-5 py-3 font-medium">Параметры</th>
                        <th className="px-5 py-3 font-medium">Отверстие</th>
                        <th className="px-5 py-3 text-right font-medium">Использований</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y">
                      {state.metadata.apertures.map((aperture) => (
                        <tr key={aperture.dCode}>
                          <td className="px-5 py-3 font-mono font-medium">D{aperture.dCode}</td>
                          <td className="px-5 py-3">
                            {apertureShapeLabel(aperture.shape, aperture.template)}
                          </td>
                          <td className="px-5 py-3 font-mono text-xs">
                            {aperture.modifiers.length ? aperture.modifiers.join(" × ") : "—"}
                          </td>
                          <td className="px-5 py-3">
                            {aperture.hasHole === null
                              ? "Зависит от макроса"
                              : aperture.hasHole
                                ? "Да"
                                : "Нет"}
                          </td>
                          <td className="px-5 py-3 text-right tabular-nums">
                            {aperture.usageCount}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              ) : (
                <p className="px-5 py-6 text-sm text-muted-foreground">
                  Определения апертур не найдены.
                </p>
              )}
            </section>
          </>
        )}
      </div>
    </main>
  )
}
