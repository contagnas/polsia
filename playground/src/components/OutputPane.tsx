import { json } from '@codemirror/lang-json'
import CodeMirror from '@uiw/react-codemirror'
import { codeMirrorTheme, errorOutputTheme } from '../cmTheme'
import type { FC } from 'react'

interface Props {
  theme: 'dark' | 'light'
  output: string
  error: boolean
}

const OutputPane: FC<Props> = ({ theme, output, error }) => (
  <div className="flex flex-col flex-1 overflow-hidden">
    <div className="flex items-center justify-between p-1 text-keyword border-b border-variable h-6 flex-none sticky top-0 z-10 bg-panel">
      <span>JSON Output</span>
    </div>
    <CodeMirror
      className="flex-1 box-border overflow-auto"
      theme={codeMirrorTheme}
      height="100%"
      value={output}
      extensions={error ? [errorOutputTheme] : [json(), codeMirrorTheme]}
      editable={false}
    />
  </div>
)

export default OutputPane
