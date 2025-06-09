import CodeMirror from '@uiw/react-codemirror'
import type { Extension } from '@codemirror/state'
import { vaporwave } from '../cmVaporwave'
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
    <div className="flex items-center justify-between p-1 border-b border-current h-6 flex-none">
      <div className="flex items-center mr-auto">
        <button
          className="border border-current bg-inherit text-inherit"
          onClick={() => onSelect(selected - 1)}
        >
          {'<'}
        </button>
        <select
          className="mx-1 ml-1 border border-current bg-inherit text-inherit"
          value={examples[selected][0]}
          onChange={(e) => {
            const idx = examples.findIndex(([n]) => n === e.target.value)
            onSelect(idx)
          }}
        >
          {examples.map(([name]) => (
            <option key={name} value={name} className="bg-inherit text-inherit">
              {name}
            </option>
          ))}
        </select>
        <button
          className="border border-current bg-inherit text-inherit"
          onClick={() => onSelect(selected + 1)}
        >
          {'>'}
        </button>
      </div>
      <label className="m-1">
        Editor:
        <select
          className="ml-2 border border-current bg-inherit text-inherit"
          value={editor}
          onChange={(e) => onEditorChange(e.target.value as any)}
        >
          <option value="basic" className="bg-inherit text-inherit">Basic</option>
          <option value="vim" className="bg-inherit text-inherit">Vim</option>
          <option value="emacs" className="bg-inherit text-inherit">Em*cs</option>
        </select>
      </label>
    </div>
    <CodeMirror
      className="flex-1 box-border overflow-auto"
      theme={theme === 'dark' ? vaporwave : 'light'}
      height="100%"
      value={src}
      extensions={theme === 'dark' ? [...vaporwave, ...extensions] : extensions}
      onChange={onChange}
    />
  </div>
)

export default EditorPane
