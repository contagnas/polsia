import type { FC } from 'react'

interface Props {
  theme: 'dark' | 'light'
  onToggle: () => void
}

const Header: FC<Props> = ({ theme, onToggle }) => (
  <header className="sticky top-0 z-20 flex items-center justify-between p-1 border-b border-variable bg-panel">
    <span className="font-bold mr-4 text-accent">POLSIA</span>
    <button
      className="m-1 self-end border text-accent px-2 py-1 hover:cursor-pointer hover:text-variable"
      onClick={onToggle}
    >
      Switch to {theme === 'dark' ? 'light' : 'dark'}
    </button>
  </header>
)

export default Header
