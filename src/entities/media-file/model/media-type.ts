export type MediaFileKind = "image" | "audio" | "video" | "unknown"

export type MediaFileInfo = {
  extension: string
  kind: MediaFileKind
}

const extensionsByKind: Record<Exclude<MediaFileKind, "unknown">, Set<string>> = {
  image: new Set([
    "jpg", "jpeg", "png", "webp", "avif", "gif", "bmp", "tif", "tiff",
    "ico", "heic", "heif", "svg",
  ]),
  audio: new Set([
    "mp3", "wav", "flac", "aac", "ogg", "oga", "opus", "m4a", "wma",
    "aif", "aiff", "alac", "amr",
  ]),
  video: new Set([
    "mp4", "mov", "mkv", "webm", "avi", "flv", "wmv", "m4v", "mpg",
    "mpeg", "3gp", "ts", "mts", "m2ts", "ogv",
  ]),
}

export const mediaFileKindLabels: Record<MediaFileKind, string> = {
  image: "Изображение",
  audio: "Аудио",
  video: "Видео",
  unknown: "Неизвестный тип",
}

export const imageOutputFormats = [
  "JPG",
  "PNG",
  "WEBP",
  "AVIF",
  "GIF",
  "BMP",
  "TIFF",
  "ICO",
] as const

export const audioOutputFormats = [
  "MP3",
  "WAV",
  "FLAC",
  "AAC",
  "OGG",
  "OPUS",
  "M4A",
  "WMA",
  "AIFF",
] as const

export const videoOutputFormats = ["MP4", "MKV", "WEBM", "MOV", "AVI"] as const

export function getFileExtension(name: string) {
  const dotIndex = name.lastIndexOf(".")

  if (dotIndex <= 0 || dotIndex === name.length - 1) return ""
  return name.slice(dotIndex + 1).toLowerCase()
}

export function detectMediaFile(name: string, mimeType?: string): MediaFileInfo {
  const extension = getFileExtension(name)
  const mimeKind = mimeType?.split("/", 1)[0]

  if (mimeKind === "image" || mimeKind === "audio" || mimeKind === "video") {
    return { extension, kind: mimeKind }
  }

  for (const kind of ["image", "audio", "video"] as const) {
    if (extensionsByKind[kind].has(extension)) return { extension, kind }
  }

  return { extension, kind: "unknown" }
}
