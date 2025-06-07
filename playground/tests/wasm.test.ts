import { expect, test } from 'vitest'
import { existsSync } from 'fs'

test('wasm package files exist', () => {
  const js = new URL('../../pkg/polsia.js', import.meta.url)
  const wasm = new URL('../../pkg/polsia_bg.wasm', import.meta.url)
  expect(existsSync(js)).toBe(true)
  expect(existsSync(wasm)).toBe(true)
})
