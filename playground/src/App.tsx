import { useEffect, useState } from 'react'
import { polsia } from './polsia'
import { vim } from '@replit/codemirror-vim'
import { emacs } from '@replit/codemirror-emacs'
import type { Extension } from '@codemirror/state'
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
  const [editor, setEditor] = useState<'basic' | 'vim' | 'emacs'>(() => {
    const saved = localStorage.getItem('editor')
    if (saved === 'basic' || saved === 'vim' || saved === 'emacs')
      return saved as any
    return 'basic'
  })
  const [output, setOutput] = useState('Loading...')
  const [selected, setSelected] = useState(DEFAULT_INDEX)
  const [src, setSrc] = useState(DEFAULT_SRC)

  function update(code: string) {
    try {
      setOutput(wasm.polsia_to_json(code))
    } catch (e) {
      setOutput('Error: ' + e)
    }
  }

  function select(idx: number) {
    const n = (idx + examples.length) % examples.length
    setSelected(n)
    const code = examples[n][1]
    setSrc(code)
    update(code)
  }

  const extensions: Extension[] = [polsia()]
  if (editor === 'vim') {
    extensions.unshift(vim({ status: true } as any))
  } else if (editor === 'emacs') {
    extensions.unshift(emacs())
  }

  let styles = getComputedStyle(document.documentElement)
  let shadow = styles.getPropertyValue('--color-accent')

  return (
    <div
      className={`flex flex-col h-full font-mono box-border overflow-hidden bg-background text-foreground ${theme === 'dark' ? 'dark' : 'light'}`}
    >
      <Header
        theme={theme}
        onToggle={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
      />
      <div className="flex flex-col md:flex-row flex-1 overflow-hidden gap-1">
        <EditorPane
          theme={theme}
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
            setSrc(v)
            update(v)
          }}
        />
        <OutputPane theme={theme} output={output} />
      </div>
      <Footer />
    </div>
  )
}

export default App
