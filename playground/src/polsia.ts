import { StreamLanguage } from '@codemirror/language'
import type { StreamParser, StringStream } from '@codemirror/language'

interface State {
  inString: boolean
  escape: boolean
  indent: number
}

const parser: StreamParser<State> = {
  startState() {
    return { inString: false, escape: false, indent: 0 }
  },

  token(stream: StringStream, state: State) {
    if (state.inString) {
      while (!stream.eol()) {
        const ch = stream.next()
        if (state.escape) {
          state.escape = false
          continue
        }
        if (ch === '\\') {
          state.escape = true
        } else if (ch === '"') {
          state.inString = false
          break
        }
      }
      return 'string'
    }

    if (stream.match(/^#.*/)) return 'comment'

    if (stream.match('"')) {
      state.inString = true
      return 'string'
    }

    if (
      stream.match(
        /\b(?:noexport|null|true|false|Any|Nothing|Int|Number|Rational|Float|String|Boolean)\b/
      )
    ) {
      return 'keyword'
    }

    if (stream.match(/-?(?:0|[1-9][\d]*)(?:\.\d+)?(?:[eE][+-]?\d+)?/)) {
      return 'number'
    }

    if (!state.inString && stream.match('{')) {
      state.indent++
      return 'bracket'
    }

    if (!state.inString && stream.match('}')) {
      state.indent = Math.max(0, state.indent - 1)
      return 'bracket'
    }

    if (!state.inString && stream.match('[')) {
      state.indent++
      return 'bracket'
    }

    if (!state.inString && stream.match(']')) {
      state.indent = Math.max(0, state.indent - 1)
      return 'bracket'
    }

    if (stream.match(/[{},\[\]]/)) return 'bracket'

    if (stream.match(/:/)) return 'operator'

    if (stream.match(/[A-Za-z_][\w.]*/)) return 'variableName'

    stream.next()
    return null
  },

  indent(state: State, textAfter: string) {
    let level = state.indent
    if (/^[\}\]]/.test(textAfter)) level--
    return Math.max(0, level) * 2
  },
}

export function polsia() {
  return StreamLanguage.define(parser)
}
