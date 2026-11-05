import { useState } from "react"
import { revealItemInDir } from "@tauri-apps/plugin-opener"
import { FolderOpenIcon, HistoryIcon, Trash2Icon } from "lucide-react"

import { getConversionHistory, saveConversionHistory, type ConversionRecord } from "@/entities/conversion-history"
import { Button } from "@/shared/ui/button"
import { Input } from "@/shared/ui/input"

export function HistoryPage() {
  const [initial] = useState(() => {
    try { return { records: getConversionHistory(), error: "" } }
    catch { return { records: [] as ConversionRecord[], error: "Не удалось прочитать историю. Можно очистить её и начать заново." } }
  })
  const [records, setRecords] = useState(initial.records)
  const [error, setError] = useState(initial.error)
  const [query, setQuery] = useState("")
  const [confirmClear, setConfirmClear] = useState(false)

  function update(next: ConversionRecord[]) {
    try {
      saveConversionHistory(next)
      setRecords(next)
      setError("")
      setConfirmClear(false)
    } catch { setError("Не удалось сохранить изменения истории.") }
  }

  async function reveal(path: string) {
    try {
      await revealItemInDir(path)
      setError("")
    } catch { setError("Не удалось показать результат в проводнике. Возможно, файл перемещён или удалён.") }
  }

  const visible = records.filter((record) =>
    `${record.inputName} ${record.outputPath} ${record.format}`.toLocaleLowerCase()
      .includes(query.trim().toLocaleLowerCase()),
  )

  return (
    <main className="min-h-0 min-w-0 flex-1 overflow-y-auto p-6">
      <div className="mx-auto flex max-w-4xl flex-col gap-6">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <h1 className="text-3xl font-semibold tracking-tight">История конвертаций</h1>
            <p className="mt-1 text-sm text-muted-foreground">Последние 200 успешных конвертаций на этом устройстве.</p>
          </div>
          <Button variant="outline" disabled={!records.length && !error} onClick={() => setConfirmClear(true)}>
            <Trash2Icon /> Очистить историю
          </Button>
        </div>
        {confirmClear && (
          <section className="flex flex-wrap items-center gap-3 rounded-xl border bg-card p-4">
            <p className="flex-1 text-sm">Удалить все записи? Исходные файлы и результаты сохранятся.</p>
            <Button variant="destructive" onClick={() => update([])}>Удалить записи</Button>
            <Button variant="outline" onClick={() => setConfirmClear(false)}>Отмена</Button>
          </section>
        )}
        {error && <p role="alert" className="text-sm text-destructive">{error}</p>}
        {records.length > 0 && <Input aria-label="Поиск по истории" placeholder="Название файла, путь или формат…" value={query} onChange={(event) => setQuery(event.target.value)} />}
        {!visible.length ? (
          <section className="rounded-xl border bg-card p-10 text-center">
            <HistoryIcon className="mx-auto mb-3 size-8 text-muted-foreground" />
            <p className="font-medium">{records.length ? "Ничего не найдено" : "История пока пуста"}</p>
            <p className="mt-1 text-sm text-muted-foreground">{records.length ? "Попробуйте другой запрос." : "Завершённые конвертации появятся здесь автоматически."}</p>
          </section>
        ) : (
          <div className="flex flex-col gap-3">
            {visible.map((record) => (
              <article key={record.id} className="rounded-xl border bg-card p-5">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="min-w-0 flex-1">
                    <h2 className="break-words font-medium">{record.inputName} → {record.format.toUpperCase()}</h2>
                    <time dateTime={record.completedAt} className="text-xs text-muted-foreground">{new Date(record.completedAt).toLocaleString("ru-RU")}</time>
                  </div>
                  <Button variant="ghost" size="icon" aria-label={`Удалить запись ${record.inputName}`} onClick={() => update(records.filter((item) => item.id !== record.id))}><Trash2Icon /></Button>
                </div>
                <dl className="mt-3 space-y-2 text-sm">
                  <div><dt className="text-muted-foreground">Исходный файл</dt><dd className="break-all">{record.inputPath}</dd></div>
                  <div><dt className="text-muted-foreground">Результат</dt><dd className="break-all">{record.outputPath}</dd></div>
                </dl>
                <Button className="mt-4" variant="outline" onClick={() => void reveal(record.outputPath)}><FolderOpenIcon /> Показать в папке</Button>
              </article>
            ))}
          </div>
        )}
      </div>
    </main>
  )
}
