import type { Extension } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import { syntaxHighlighting, HighlightStyle } from '@codemirror/language'
import { tags as t } from '@lezer/highlight'

const miamiViceTheme = EditorView.theme(
  {
    '&': {
      color: '#55f6ff',
      backgroundColor: '#000',
    },
    '.cm-content': {
      caretColor: '#ff5c93',
    },
    '&.cm-focused .cm-cursor': {
      borderLeftColor: '#ff5c93',
    },
    '&.cm-focused .cm-selectionBackground, ::selection': {
      backgroundColor: '#ff5c9344',
    },
    '.cm-gutters': {
      backgroundColor: '#000',
      color: '#55f6ff',
      border: 'none',
    },
  },
  { dark: true }
)

const miamiViceHighlight = HighlightStyle.define([
  { tag: [t.keyword, t.operator], color: '#55f6ff' },
  { tag: [t.string, t.special(t.string)], color: '#ff5c93' },
  { tag: [t.number, t.bool], color: '#f9f871' },
  { tag: t.comment, color: '#888' },
  { tag: t.variableName, color: '#55f6ff' },
  { tag: t.bracket, color: '#f9f871' },
])

export const miamiVice: Extension[] = [miamiViceTheme, syntaxHighlighting(miamiViceHighlight)]
