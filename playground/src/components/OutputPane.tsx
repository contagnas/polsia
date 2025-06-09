import { json } from '@codemirror/lang-json'
import CodeMirror from '@uiw/react-codemirror'
import { consoleDark } from '@uiw/codemirror-theme-console'
import type { FC } from 'react'

interface Props {
  theme: 'dark' | 'light'
  output: string
}

const OutputPane: FC<Props> = ({ theme, output }) => (
  <div className="flex flex-col flex-1 overflow-hidden">
    <div className="flex items-center justify-between p-1 border-b border-current h-6 flex-none">
      <span>JSON Output</span>
    </div>
    <CodeMirror
      className="flex-1 box-border overflow-auto"
      theme={theme === 'dark' ? consoleDark : 'light'}
      height="100%"
      value={output}
      extensions={[json()]}
      editable={false}
    />
  </div>
)

export default OutputPane
