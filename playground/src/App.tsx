import { useEffect, useState } from 'react'
import CodeMirror from '@uiw/react-codemirror'
import { polsia } from './polsia'
import { json } from '@codemirror/lang-json'
import { vim } from '@replit/codemirror-vim'
import { emacs } from '@replit/codemirror-emacs'
import type { Extension } from '@codemirror/state'
import Marquee from 'react-fast-marquee'
import './index.css'
import * as wasm from 'polsia'
const modules = import.meta.glob('../../examples/*.pls', { eager: true, as: 'raw' }) as Record<string, string>
const examples = Object.entries(modules)
  .map(([path, code]) => [path.split('/').pop()!, code] as const)
  .sort(([a], [b]) => a.localeCompare(b))

const DEFAULT_INDEX = 0
const DEFAULT_SRC = examples[DEFAULT_INDEX][1]

function App() {
  const [theme, setTheme] = useState<'dark' | 'light'>('dark')
  const [editor, setEditor] = useState<'basic' | 'vim' | 'emacs'>(() => {
    const saved = localStorage.getItem('editor')
    if (saved === 'basic' || saved === 'vim' || saved === 'emacs') return saved as any
    return 'basic'
  })
  const [output, setOutput] = useState('Loading...')
  const [selected, setSelected] = useState(DEFAULT_INDEX)
  const [src, setSrc] = useState(DEFAULT_SRC)

  useEffect(() => {
    ;(async () => {
      setOutput(wasm.polsia_to_json(src))
    })()
  }, [])

  function update(src: string) {
    try {
      setOutput(wasm.polsia_to_json(src))
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
    extensions.unshift(
      vim({ status: true } as any)
    )
  } else if (editor === 'emacs') {
    extensions.unshift(emacs())
  }

  return (
    <div className={`app theme-${theme}`}>
      <header className="header">
        <span className="title">POLSIA</span>
        <button
          className="switcher"
          onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
        >
          Switch to {theme === 'dark' ? 'light' : 'dark'}
        </button>
      </header>
      <div className="content">
        <div className="editor">
          <div className="section-header">
            <div className="example">
              <button onClick={() => select(selected - 1)}>{'<'}</button>
              <select
                value={examples[selected][0]}
                onChange={(e) => {
                  const idx = examples.findIndex(([n]) => n === e.target.value)
                  select(idx)
                }}
              >
                {examples.map(([name]) => (
                  <option key={name} value={name}>
                    {name}
                  </option>
                ))}
              </select>
              <button onClick={() => select(selected + 1)}>{'>'}</button>
            </div>
            <label className="editor-select">
              Editor:
              <select
                value={editor}
                onChange={(e) => {
                  const val = e.target.value as 'basic' | 'vim' | 'emacs'
                  setEditor(val)
                  localStorage.setItem('editor', val)
                }}
              >
                <option value="basic">Basic</option>
                <option value="vim">Vim</option>
                <option value="emacs">Em*cs</option>
              </select>
            </label>
          </div>
          <CodeMirror
            className="pane"
            theme={theme}
            height="100%"
            value={src}
            extensions={extensions}
            onChange={(v) => {
              setSrc(v)
              update(v)
            }}
          />
        </div>
        <div className="output">
          <div className="section-header">
            <span>JSON Output</span>
          </div>
          <CodeMirror
            className="pane"
            theme={theme}
            height="100%"
            value={output}
            extensions={[json()]}
            editable={false}
          />
        </div>
      </div>
      <footer className="footer">
        <Marquee autoFill>POLSIA &nbsp;</Marquee>
      </footer>
    </div>
  )
}

export default App
