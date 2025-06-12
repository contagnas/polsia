export const editorModes = ['basic', 'vim', 'emacs'] as const;
export type EditorMode = typeof editorModes[number];

export function isEditorMode(x: unknown): x is EditorMode {
  return (editorModes as readonly string[]).includes(x as string);
}
