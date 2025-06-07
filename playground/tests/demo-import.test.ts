import { expect, test } from 'vitest'
import demoSrc from '../../examples/demo.pls?raw'

test('demo source file loads', () => {
  expect(typeof demoSrc).toBe('string')
  expect(demoSrc.length).toBeGreaterThan(0)
})
