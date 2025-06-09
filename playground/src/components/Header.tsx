import type { FC } from 'react'

interface Props {
  theme: 'dark' | 'light'
  onToggle: () => void
}

const Header: FC<Props> = ({ theme, onToggle }) => (
  <header className="flex items-center justify-between p-1 border-b border-current">
    <span className="font-bold mr-4 text-pink-500">POLSIA</span>
    <button
      className="m-1 self-end border border-current bg-inherit text-inherit px-2 py-1 hover:cursor-pointer hover:text-pink-500"
      onClick={onToggle}
    >
      Switch to {theme === 'dark' ? 'light' : 'dark'}
    </button>
  </header>
)

export default Header
