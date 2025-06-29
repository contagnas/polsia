import { useEffect, useState, useRef } from 'react'
import { polsia } from './polsia'
import { vim } from '@replit/codemirror-vim'
import { emacs } from '@replit/codemirror-emacs'
import type { Extension } from '@codemirror/state'
import { isEditorMode, type EditorMode } from './editor'
import * as wasm from 'polsia'
import Header from './components/Header'
import Footer from './components/Footer'
import EditorPane from './components/EditorPane'
import OutputPane from './components/OutputPane'

const modules = import.meta.glob('../../examples/*.pls', {
  eager: true,
  as: 'raw',
}) as Record<string, string>
const examples = Object.entries(modules)
  .map(([path, code]) => [path.split('/').pop()!, code] as const)
  .sort(([a], [b]) => a.localeCompare(b))

const DEFAULT_INDEX = 0
const DEFAULT_SRC = examples[DEFAULT_INDEX][1]

function App() {
  const [theme, setTheme] = useState<'dark' | 'light'>('dark')
  const [editor, setEditor] = useState<EditorMode>(() => {
    const saved = localStorage.getItem('editor')
    if (isEditorMode(saved)) return saved
    return 'basic'
  })
  const [output, setOutput] = useState('Loading...')
  const [error, setError] = useState(false)
  const [selected, setSelected] = useState(DEFAULT_INDEX)
  const srcRef = useRef(DEFAULT_SRC)
  const [src, setSrc] = useState(srcRef.current)

  function update(code: string) {
    try {
      setOutput(wasm.polsia_to_json(code))
      setError(false)
    } catch (e) {
      setOutput(String(e))
      setError(true)
    }
  }

  useEffect(() => {
    update(src)
  }, [src])

  function select(idx: number) {
    const n = (idx + examples.length) % examples.length
    setSelected(n)
    const code = examples[n][1]
    setSrc(code)
    update(code)
  }

  const extensions: Extension[] = [polsia()]
  if (editor === 'vim') {
    extensions.unshift(vim({ status: true }))
  } else if (editor === 'emacs') {
    extensions.unshift(emacs())
  }

  return (
    <div
      className={`flex flex-col h-full font-mono box-border overflow-hidden bg-background text-foreground scrollbar ${theme === 'dark' ? 'dark' : 'light'}`}
    >
      <Header
        theme={theme}
        onToggle={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
      />
      <div className="flex flex-col md:flex-row flex-1 overflow-hidden gap-1">
        <EditorPane
          src={src}
          examples={examples}
          selected={selected}
          onSelect={select}
          editor={editor}
          onEditorChange={(v) => {
            setEditor(v)
            localStorage.setItem('editor', v)
          }}
          extensions={extensions}
          onChange={(v) => {
            // https://github.com/uiwjs/react-codemirror/issues/700
            srcRef.current = v
            update(v)
          }}
        />
        <OutputPane output={output} error={error} />
      </div>
      <Footer error={error} />
    </div>
  )
}

export default App
