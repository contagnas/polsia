import { useEffect, useRef, useState } from 'react'
import CodeMirror from '@uiw/react-codemirror'
import { javascript } from '@codemirror/lang-javascript'
import { json } from '@codemirror/lang-json'
import { vim } from '@replit/codemirror-vim'
import { emacs } from '@replit/codemirror-emacs'
import type { Extension } from '@codemirror/state'
import Marquee from 'react-fast-marquee'
import './index.css'
import * as wasm from 'polsia'
import demoSrc from '../../examples/demo.pls?raw'

const DEFAULT_SRC = demoSrc

function App() {
  const [theme, setTheme] = useState<'dark' | 'light'>('dark')
  const [power, setPower] = useState<'low' | 'medium' | 'high'>('medium')
  const [output, setOutput] = useState('Loading...')
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

  const extensions: Extension[] = [javascript()]
  if (power === 'high') {
    extensions.unshift(
      vim({ status: true } as any)
    )
  } else if (power === 'low') {
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
            <span>Polsia Input</span>
            <label className="power">
              power level:
              <select
                value={power}
                onChange={(e) => setPower(e.target.value as any)}
              >
                <option value="low">low (emacs)</option>
                <option value="medium">medium (basic)</option>
                <option value="high">high (vim)</option>
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
