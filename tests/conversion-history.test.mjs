import assert from "node:assert/strict"
import { test } from "node:test"
import { addConversionRecord, getConversionHistory, saveConversionHistory } from "../src/entities/conversion-history/index.ts"

test("history persists newest results, caps records and supports deletion", () => {
  let stored = null
  globalThis.localStorage = {
    getItem: () => stored,
    setItem: (_key, value) => { stored = value },
  }
  assert.deepEqual(getConversionHistory(), [])
  for (let index = 0; index < 205; index++) {
    addConversionRecord({
      id: String(index), inputName: "song.wav", inputPath: "C:/song.wav",
      outputPath: `C:/song_${index}.mp3`, format: "MP3",
      completedAt: new Date().toISOString(),
    })
  }
  assert.equal(getConversionHistory().length, 200)
  assert.equal(getConversionHistory()[0].id, "204")
  saveConversionHistory(getConversionHistory().filter((record) => record.id !== "204"))
  assert.equal(getConversionHistory()[0].id, "203")
  stored = "broken"
  assert.throws(getConversionHistory)
  saveConversionHistory([])
  assert.deepEqual(getConversionHistory(), [])
  globalThis.localStorage.setItem = () => { throw new Error("Storage unavailable") }
  assert.throws(() => saveConversionHistory([]))
})
