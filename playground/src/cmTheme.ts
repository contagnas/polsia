import {
  EditorView,
  highlightSpecialChars,
  drawSelection,
  highlightActiveLine,
  dropCursor,
} from '@codemirror/view'
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { tags as t } from '@lezer/highlight'
import { Extension } from '@codemirror/state'

// 1) The editor chrome (background, gutters, cursor, selection…)
export const cmTheme = EditorView.theme(
  {
    '&': {
      color: 'var(--foreground)',
      backgroundColor: 'var(--panel)',
    },
    '.cm-content': {
      caretColor: 'var(--foreground)',
    },
    '.cm-cursor, .cm-dropCursor': {
      borderLeftColor: 'var(--foreground)',
    },
    '&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection':
      {
        backgroundColor: 'var(--accent)',
      },
    '.cm-panels': {
      backgroundColor: 'var(--background)',
      color: 'var(--foreground)',
    },
    '.cm-gutters': {
      backgroundColor: 'var(--panel)',
      color: 'var(--comment)',
      border: 'none',
    },
    '.cm-activeLine': {
      backgroundColor: 'var(--panel)',
    },
    '.cm-activeLineGutter': {
      backgroundColor: 'var(--panel)',
    },
  },
  { dark: false }
)

// 2) The syntax highlighting rules
export const cmHighlightStyle = HighlightStyle.define([
  { tag: t.keyword, color: 'var(--keyword)', fontWeight: 'bold' },
  { tag: [t.string, t.special(t.string)], color: 'var(--string)' },
  { tag: t.number, color: 'var(--number)' },
  { tag: t.comment, color: 'var(--comment)', fontStyle: 'italic' },
  { tag: t.variableName, color: 'var(--variable)' },
  { tag: t.definition(t.variableName), color: 'var(--accent)' },
  { tag: t.typeName, color: 'var(--accent)' },
  { tag: t.operator, color: 'var(--accent)' },
  { tag: t.function(t.variableName), color: 'var(--accent)' },
  // you can add more tags here…
])

// 3) Export the array of extensions to load
export const codeMirrorTheme: Extension = [
  cmTheme,
  highlightSpecialChars(),
  drawSelection(),
  dropCursor(),
  highlightActiveLine(),
  syntaxHighlighting(cmHighlightStyle),
]
