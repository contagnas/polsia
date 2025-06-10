import CodeMirror from '@uiw/react-codemirror'
import type { Extension } from '@codemirror/state'
import { codeMirrorTheme } from '../cmTheme'
import type { FC } from 'react'

interface Props {
  theme: 'dark' | 'light'
  src: string
  examples: readonly (readonly [string, string])[]
  selected: number
  onSelect: (idx: number) => void
  editor: 'basic' | 'vim' | 'emacs'
  onEditorChange: (v: 'basic' | 'vim' | 'emacs') => void
  extensions: Extension[]
  onChange: (src: string) => void
}

const EditorPane: FC<Props> = ({
  theme,
  src,
  examples,
  selected,
  onSelect,
  editor,
  onEditorChange,
  extensions,
  onChange,
}) => (
  <div className="flex flex-col flex-1 overflow-hidden">
    <div className="flex items-center justify-between p-1 border-b text-keyword border-variable h-6 flex-none sticky top-0 z-10 bg-panel">
      <div className="flex items-center mr-auto">
        <button
          className="border border-variable bg-background text-fg w-5 h-5 flex items-center justify-center"
          onClick={() => onSelect(selected - 1)}
        >
          {'<'}
        </button>
        <select
          className="mx-1 ml-1 border border-variable bg-background text-fg h-5"
          value={examples[selected][0]}
          onChange={(e) => {
            const idx = examples.findIndex(([n]) => n === e.target.value)
            onSelect(idx)
          }}
        >
          {examples.map(([name]) => (
            <option key={name} value={name} className="bg-background text-fg">
              {name}
            </option>
          ))}
        </select>
        <button
          className="border border-variable bg-background text-fg w-5 h-5 flex items-center justify-center"
          onClick={() => onSelect(selected + 1)}
        >
          {'>'}
        </button>
      </div>
      <label className="m-1">
        Editor:
        <select
          className="ml-2 border border-variable bg-background text-fg"
          value={editor}
          onChange={(e) => onEditorChange(e.target.value as any)}
        >
          <option value="basic" className="bg-background text-fg">
            Basic
          </option>
          <option value="vim" className="bg-background text-fg">
            Vim
          </option>
          <option value="emacs" className="bg-background text-fg">
            Em*cs
          </option>
        </select>
      </label>
    </div>
    <CodeMirror
      className="flex-1 box-border overflow-auto"
      theme={codeMirrorTheme}
      height="100%"
      value={src}
      extensions={[codeMirrorTheme, ...extensions]}
      onChange={onChange}
    />
  </div>
)

export default EditorPane
