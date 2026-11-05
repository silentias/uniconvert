import assert from "node:assert/strict"
import { test } from "node:test"
import { estimateProgress, formatElapsed } from "../src/shared/lib/conversion-progress.ts"

test("estimates from measured progress and hides unreliable or stale estimates", () => {
  assert.equal(estimateProgress([], 0), null)
  assert.equal(estimateProgress([{ seconds: 0, percent: 0 }, { seconds: 1, percent: 10 }], 1), null)
  const samples = [{ seconds: 2, percent: 10 }, { seconds: 12, percent: 30 }]
  assert.deepEqual(estimateProgress(samples, 12), { percentPerMinute: 120, remainingSeconds: 35 })
  assert.equal(estimateProgress(samples, 23), null)
  assert.equal(estimateProgress([{ seconds: 0, percent: 10 }, { seconds: 10, percent: 10 }], 10), null)
  assert.equal(estimateProgress([{ seconds: 0, percent: 10 }, { seconds: 10, percent: 99.5 }], 10), null)
  assert.equal(formatElapsed(3661.9), "1:01:01")
  assert.equal(formatElapsed(9), "0:09")
})
