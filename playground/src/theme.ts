const hexToRgb = (hex: string): string => {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex)
  if (!result) throw new Error(`Invalid hex color: ${hex}`)
  return `${parseInt(result[1], 16)} ${parseInt(result[2], 16)} ${parseInt(result[3], 16)}`
}

export const darkColors = {
  bg: hexToRgb('#0d1117'),
  panel: hexToRgb('#161b22'),
  fg: hexToRgb('#c9d1d9'),
  accent: hexToRgb('#58a6ff'),
  keyword: hexToRgb('#ff7b72'),
  string: hexToRgb('#a5d6ff'),
  number: hexToRgb('#d29922'),
  variable: hexToRgb('#d2a8ff'),
  comment: hexToRgb('#8b949e'),
} as const

console.log('darkcolors', darkColors)

export const lightColors = {
  bg: hexToRgb('#ffffff'),
  panel: hexToRgb('#f6f8fa'),
  fg: hexToRgb('#24292f'),
  accent: hexToRgb('#0969da'),
  keyword: hexToRgb('#d73a49'),
  string: hexToRgb('#032f62'),
  number: hexToRgb('#005cc5'),
  variable: hexToRgb('#8250df'),
  comment: hexToRgb('#6e7781'),
} as const

export const applyTheme = (isDark: boolean) => {
  const colors = isDark ? darkColors : lightColors
  const root = document.documentElement

  Object.entries(colors).forEach(([key, value]) => {
    root.style.setProperty(`--color-${key}`, value)
  })
}
