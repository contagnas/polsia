import type { Extension } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import { syntaxHighlighting, HighlightStyle } from '@codemirror/language'
import { tags as t } from '@lezer/highlight'
import { darkColors, lightColors } from './theme'

const darkTheme = EditorView.theme(
  {
    '&': { color: darkColors.cyan, backgroundColor: darkColors.bg },
    '.cm-content': { caretColor: darkColors.pink },
    '&.cm-focused .cm-cursor': { borderLeftColor: darkColors.pink },
    '&.cm-focused .cm-selectionBackground, ::selection': {
      backgroundColor: `${darkColors.pink}44`,
    },
    '.cm-gutters': {
      backgroundColor: darkColors.bg,
      color: darkColors.purple,
      border: 'none',
    },
  },
  { dark: true },
)

const darkHighlight = HighlightStyle.define([
  { tag: t.keyword, color: darkColors.orange },
  { tag: t.operator, color: darkColors.pink },
  { tag: [t.string, t.special(t.string)], color: darkColors.cyan },
  { tag: [t.number, t.bool], color: darkColors.orange },
  { tag: t.comment, color: darkColors.comment },
  { tag: t.variableName, color: darkColors.purple },
  { tag: t.bracket, color: darkColors.cyan },
])

export const cmDark: Extension[] = [darkTheme, syntaxHighlighting(darkHighlight)]

const lightTheme = EditorView.theme({
  '&': { color: lightColors.purple, backgroundColor: lightColors.bg },
  '.cm-content': { caretColor: lightColors.red },
  '&.cm-focused .cm-cursor': { borderLeftColor: lightColors.red },
  '&.cm-focused .cm-selectionBackground, ::selection': {
    backgroundColor: `${lightColors.red}44`,
  },
  '.cm-gutters': {
    backgroundColor: lightColors.bg,
    color: lightColors.navy,
    border: 'none',
  },
})

const lightHighlight = HighlightStyle.define([
  { tag: t.keyword, color: lightColors.magenta },
  { tag: t.operator, color: lightColors.red },
  { tag: [t.string, t.special(t.string)], color: lightColors.pink },
  { tag: [t.number, t.bool], color: lightColors.magenta },
  { tag: t.comment, color: lightColors.comment },
  { tag: t.variableName, color: lightColors.navy },
  { tag: t.bracket, color: lightColors.pink },
])

export const cmLight: Extension[] = [lightTheme, syntaxHighlighting(lightHighlight)]
