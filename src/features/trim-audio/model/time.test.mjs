import assert from "node:assert/strict"
import test from "node:test"
import { formatAudioTime, parseAudioTime } from "./time.ts"

test("accepts seconds and precise timestamps, including recordings over an hour", () => {
  assert.equal(parseAudioTime("1:02:03.125"), 3723.125)
  assert.equal(parseAudioTime("62:03,125"), 3723.125)
  assert.equal(parseAudioTime(" 12.005 "), 12.005)
  assert.equal(formatAudioTime(59.9996), "01:00.000")
  assert.equal(parseAudioTime(formatAudioTime(3723.125)), 3723.125)
})

test("rejects malformed timestamps instead of silently changing the selection", () => {
  for (const text of ["", "-1", "1:60", "1:60:00", "Infinity", "abc", "1.0001", "1::2", "1:2:3:4"]) {
    assert.equal(parseAudioTime(text), null, text)
  }
})
