import { useId, useState } from "react"
import { FolderOpenIcon, XIcon } from "lucide-react"
import { open } from "@tauri-apps/plugin-dialog"

import { Button } from "@/shared/ui/button"
import { Input } from "@/shared/ui/input"

type OutputDirectoryPickerProps = {
  value: string
  onChange: (path: string) => void
  description?: string
}

export function OutputDirectoryPicker({
  value,
  onChange,
  description,
}: OutputDirectoryPickerProps) {
  const inputId = useId()
  const [error, setError] = useState("")

  async function selectDirectory() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: value || undefined,
        title: "Выберите папку для готовых файлов",
      })

      if (selected) {
        onChange(selected)
        setError("")
      }
    } catch {
      setError("Не удалось открыть окно выбора папки.")
    }
  }

  return (
    <div className="space-y-2">
      <div className="space-y-1">
        <label htmlFor={inputId} className="text-sm font-medium">
          Папка для готовых файлов
        </label>
        {description && (
          <p className="text-sm text-muted-foreground">{description}</p>
        )}
      </div>

      <div className="flex gap-2">
        <Input
          id={inputId}
          value={value}
          readOnly
          placeholder="Не выбрана"
          className="min-w-0 flex-1"
          title={value || undefined}
        />
        {value && (
          <Button
            type="button"
            variant="outline"
            size="icon"
            aria-label="Сбросить выбранную папку"
            onClick={() => onChange("")}
          >
            <XIcon />
          </Button>
        )}
        <Button type="button" variant="outline" onClick={selectDirectory}>
          <FolderOpenIcon data-icon="inline-start" />
          Выбрать
        </Button>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}
    </div>
  )
}
