import { EditorView } from '@codemirror/view'
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { tags as t } from '@lezer/highlight'
import { Extension } from '@codemirror/state'

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
      backgroundColor: 'rgba(0, 0, 0, 0)',
    },
    '.cm-activeLineGutter': {
      backgroundColor: 'var(--panel)',
    },
    ".cm-selectionBackground": {
      background: `color-mix(
          in srgb,
          var(--keyword) 15%,
          transparent
      );`
    },
    "&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground": {
      background:`color-mix(
        in srgb,
        var(--variable) 15%,
        transparent
      );`

    },

  },
  { dark: false }
)

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
])

export const codeMirrorTheme: Extension = [
  cmTheme,
  syntaxHighlighting(cmHighlightStyle),
]

export const errorOutputTheme: Extension = [
  EditorView.theme({
    '.cm-content': { color: 'var(--keyword)' },
  }),
]
