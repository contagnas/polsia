import type { Config } from 'tailwindcss'
import { darkColors, lightColors } from './src/theme'

export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        dark: darkColors,
        light: lightColors,
      },
    },
  },
  darkMode: 'class',
  plugins: [],
} satisfies Config
