import { useState } from "react"

import {
  getDefaultOutputDirectory,
  saveDefaultOutputDirectory,
} from "@/entities/app-settings"
import { OutputDirectoryPicker } from "@/features/select-output-directory"
import { ThemeSwitch } from "@/features/toggle-theme"

export function SettingsPage() {
  const [outputDirectory, setOutputDirectory] = useState(
    getDefaultOutputDirectory,
  )

  function updateOutputDirectory(path: string) {
    setOutputDirectory(path)
    saveDefaultOutputDirectory(path)
  }

  return (
    <main className="min-h-0 min-w-0 flex-1 overflow-y-auto p-6">
      <div className="flex flex-col gap-6">
      <div>
        <h1 className="text-3xl font-semibold tracking-tight">Настройки</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Настройте внешний вид и поведение приложения.
        </p>
      </div>

      <section className="max-w-2xl rounded-xl border bg-card text-card-foreground">
        <div className="flex items-center justify-between gap-6 p-5">
          <div className="space-y-1">
            <h2 className="font-medium">Тёмная тема</h2>
            <p className="text-sm text-muted-foreground">
              Использовать тёмное оформление интерфейса.
            </p>
          </div>
          <ThemeSwitch />
        </div>
      </section>

      <section className="max-w-2xl rounded-xl border bg-card p-5 text-card-foreground">
        <OutputDirectoryPicker
          value={outputDirectory}
          onChange={updateOutputDirectory}
          description="Эта папка будет автоматически подставляться при новой конвертации. Настройка сохраняется на этом устройстве."
        />
      </section>
      </div>
    </main>
  )
}
