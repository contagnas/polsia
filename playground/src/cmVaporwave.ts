import type { Extension } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import { syntaxHighlighting, HighlightStyle } from '@codemirror/language'
import { tags as t } from '@lezer/highlight'

const vaporwaveTheme = EditorView.theme(
  {
    '&': {
      color: '#b967ff',
      backgroundColor: '#000',
    },
    '.cm-content': {
      caretColor: '#ff71ce',
    },
    '&.cm-focused .cm-cursor': {
      borderLeftColor: '#ff71ce',
    },
    '&.cm-focused .cm-selectionBackground, ::selection': {
      backgroundColor: '#ff71ce44',
    },
    '.cm-gutters': {
      backgroundColor: '#000',
      color: '#b967ff',
      border: 'none',
    },
  },
  { dark: true }
)

const vaporwaveHighlight = HighlightStyle.define([
  { tag: [t.keyword, t.operator], color: '#b967ff' },
  { tag: [t.string, t.special(t.string)], color: '#ff71ce' },
  { tag: [t.number, t.bool], color: '#00eaff' },
  { tag: t.comment, color: '#888' },
  { tag: t.variableName, color: '#b967ff' },
  { tag: t.bracket, color: '#00eaff' },
])

export const vaporwave: Extension[] = [vaporwaveTheme, syntaxHighlighting(vaporwaveHighlight)]
