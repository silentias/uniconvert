export type ConversionRecord = {
  id: string
  inputName: string
  inputPath: string
  outputPath: string
  format: string
  completedAt: string
}

const storageKey = "uniconvert.conversion-history.v1"
const limit = 200

function isRecord(value: unknown): value is ConversionRecord {
  if (!value || typeof value !== "object") return false
  return ["id", "inputName", "inputPath", "outputPath", "format", "completedAt"]
    .every((key) => typeof Reflect.get(value, key) === "string")
    && Number.isFinite(Date.parse(Reflect.get(value, "completedAt")))
}

export function getConversionHistory(): ConversionRecord[] {
  const raw = localStorage.getItem(storageKey)
  if (!raw) return []
  const value: unknown = JSON.parse(raw)
  if (!Array.isArray(value) || !value.every(isRecord)) {
    throw new Error("Не удалось прочитать сохранённую историю.")
  }
  return value.slice(0, limit)
}

export function saveConversionHistory(records: ConversionRecord[]) {
  localStorage.setItem(storageKey, JSON.stringify(records.slice(0, limit)))
}

export function addConversionRecord(record: ConversionRecord) {
  saveConversionHistory([record, ...getConversionHistory()])
}
