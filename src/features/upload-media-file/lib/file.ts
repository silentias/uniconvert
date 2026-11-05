import type { UploadedFile } from "../model/types"

export function fromBrowserFile(
  file: File,
  source: "picker" | "browser-drop",
): UploadedFile {
  return {
    source,
    name: file.name,
    size: file.size,
    file,
  }
}

export function fromTauriPath(path: string): UploadedFile {
  const name = path.split(/[\\/]/).pop() || path

  return {
    source: "tauri-drop",
    name,
    path,
  }
}

export function formatFileSize(size?: number) {
  if (size === undefined) return "Размер определится при обработке"
  if (size === 0) return "0 Б"

  const units = ["Б", "КБ", "МБ", "ГБ"]
  const unitIndex = Math.min(
    Math.floor(Math.log(size) / Math.log(1024)),
    units.length - 1,
  )

  return `${(size / 1024 ** unitIndex).toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`
}
