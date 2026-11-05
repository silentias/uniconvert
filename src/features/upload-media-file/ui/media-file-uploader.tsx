import { useEffect, useRef, useState, type DragEvent } from "react"
import { FileIcon, UploadCloudIcon, XIcon } from "lucide-react"
import { getCurrentWebview } from "@tauri-apps/api/webview"
import { open } from "@tauri-apps/plugin-dialog"

import { detectMediaFile, mediaFileKindLabels } from "@/entities/media-file"
import { Button } from "@/shared/ui/button"
import { Input } from "@/shared/ui/input"
import { cn } from "@/shared/lib/cn"

import { formatFileSize, fromBrowserFile, fromTauriPath } from "../lib/file"
import type { UploadedFile } from "../model/types"

type MediaFileUploaderProps = {
  disabled?: boolean
  onFileChange?: (file: UploadedFile | null) => void
  accept?: string
  dialogExtensions?: string[]
  dialogTitle?: string
  description?: string
  fileTypeLabel?: string
}

export function MediaFileUploader({
  onFileChange,
  disabled,
  accept,
  dialogExtensions,
  dialogTitle = "Выберите файл для конвертации",
  description = "Пока поддерживаются файлы любого типа. Можно также выбрать файл вручную.",
  fileTypeLabel,
}: MediaFileUploaderProps) {
  const inputRef = useRef<HTMLInputElement>(null)
  const [uploadedFile, setUploadedFile] = useState<UploadedFile | null>(null)
  const [isDragging, setIsDragging] = useState(false)

  useEffect(() => {
    if (disabled || !("__TAURI_INTERNALS__" in window)) return

    let unlisten: (() => void) | undefined
    let disposed = false

    void getCurrentWebview()
      .onDragDropEvent(({ payload }) => {
        if (payload.type === "enter" || payload.type === "over") {
          setIsDragging(true)
        }

        if (payload.type === "leave") {
          setIsDragging(false)
        }

        if (payload.type === "drop") {
          setIsDragging(false)
          const [path] = payload.paths
          if (path) {
            const file = fromTauriPath(path)
            setUploadedFile(file)
            onFileChange?.(file)
          }
        }
      })
      .then((stopListening) => {
        if (disposed) stopListening()
        else unlisten = stopListening
      })

    return () => {
      disposed = true
      unlisten?.()
    }
  }, [onFileChange, disabled])

  function handleDrop(event: DragEvent<HTMLDivElement>) {
    event.preventDefault()
    if (disabled) return
    setIsDragging(false)

    const [file] = Array.from(event.dataTransfer.files)
    if (file) {
      const uploaded = fromBrowserFile(file, "browser-drop")
      setUploadedFile(uploaded)
      onFileChange?.(uploaded)
    }
  }

  function clearFile() {
    setUploadedFile(null)
    onFileChange?.(null)
    if (inputRef.current) inputRef.current.value = ""
  }

  async function selectFile() {
    if (disabled) return
    if (!("__TAURI_INTERNALS__" in window)) {
      inputRef.current?.click()
      return
    }

    const path = await open({
      multiple: false,
      directory: false,
      title: dialogTitle,
      filters: dialogExtensions?.length
        ? [{ name: "Поддерживаемые файлы", extensions: dialogExtensions }]
        : undefined,
    })

    if (path) {
      const uploaded = fromTauriPath(path)
      setUploadedFile(uploaded)
      onFileChange?.(uploaded)
    }
  }

  if (uploadedFile) {
    const mediaInfo = detectMediaFile(uploadedFile.name, uploadedFile.file?.type)

    return (
      <section className="flex items-center gap-4 rounded-xl border bg-card p-4 text-card-foreground shadow-xs">
        <div className="flex size-11 shrink-0 items-center justify-center rounded-lg bg-muted">
          <FileIcon className="size-5 text-muted-foreground" />
        </div>
        <div className="min-w-0 flex-1">
          <p className="truncate text-sm font-medium">{uploadedFile.name}</p>
          <p className="truncate text-xs text-muted-foreground">
            {uploadedFile.path ?? formatFileSize(uploadedFile.size)}
          </p>
          <p className="mt-1 text-xs font-medium text-foreground/70">
            {fileTypeLabel ?? mediaFileKindLabels[mediaInfo.kind]}
            {mediaInfo.extension && ` · .${mediaInfo.extension.toUpperCase()}`}
          </p>
        </div>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          aria-label="Удалить файл"
          disabled={disabled}
          onClick={clearFile}
        >
          <XIcon />
        </Button>
      </section>
    )
  }

  return (
    <section>
      <Input
        ref={inputRef}
        type="file"
        accept={accept}
        className="sr-only"
        onChange={(event) => {
          const file = event.target.files?.[0]
          if (file) {
            const uploaded = fromBrowserFile(file, "picker")
            setUploadedFile(uploaded)
            onFileChange?.(uploaded)
          }
        }}
      />

      <div
        role="button"
        tabIndex={0}
        onClick={() => void selectFile()}
        onKeyDown={(event) => {
          if (event.key === "Enter" || event.key === " ") {
            event.preventDefault()
            void selectFile()
          }
        }}
        onDragEnter={(event) => {
          event.preventDefault()
          setIsDragging(true)
        }}
        onDragOver={(event) => event.preventDefault()}
        onDragLeave={(event) => {
          if (!event.currentTarget.contains(event.relatedTarget as Node | null)) {
            setIsDragging(false)
          }
        }}
        onDrop={handleDrop}
        className={cn(
          "flex min-h-72 cursor-pointer flex-col items-center justify-center rounded-xl border border-dashed bg-card p-8 text-center text-card-foreground transition-colors outline-none",
          "hover:border-foreground/30 hover:bg-muted/40 focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50",
          isDragging && "border-primary bg-primary/5 ring-3 ring-primary/15",
        )}
      >
        <div className="mb-4 flex size-12 items-center justify-center rounded-xl bg-muted">
          <UploadCloudIcon className="size-6 text-muted-foreground" />
        </div>
        <h2 className="text-base font-medium">
          {isDragging ? "Отпустите файл здесь" : "Перетащите файл сюда"}
        </h2>
        <p className="mt-1 max-w-sm text-sm text-muted-foreground">
          {description}
        </p>
        <Button
          type="button"
          variant="outline"
          className="mt-5"
          onClick={(event) => {
            event.stopPropagation()
            void selectFile()
          }}
        >
          Выбрать файл
        </Button>
      </div>
    </section>
  )
}
