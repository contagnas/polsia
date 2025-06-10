import { expect, test } from 'vitest'
const modules = import.meta.glob('../../examples/*.pls', {
  eager: true,
  as: 'raw',
}) as Record<string, string>

test('example source files load', () => {
  const files = Object.values(modules)
  expect(files.length).toBeGreaterThan(0)
  for (const src of files) {
    expect(typeof src).toBe('string')
    expect(src.length).toBeGreaterThan(0)
  }
})
