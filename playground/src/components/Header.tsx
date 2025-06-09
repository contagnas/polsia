import type { FC } from 'react'

interface Props {
  theme: 'dark' | 'light'
  onToggle: () => void
}

const Header: FC<Props> = ({ theme, onToggle }) => (
  <header className="sticky top-0 z-20 flex items-center justify-between p-1 border-b border-current bg-inherit">
    <span className="font-bold mr-4 dark:text-dark-pink text-light-magenta">POLSIA</span>
    <button
      className="m-1 self-end border border-current bg-inherit text-inherit px-2 py-1 hover:cursor-pointer hover:dark:text-dark-orange hover:text-light-red"
      onClick={onToggle}
    >
      Switch to {theme === 'dark' ? 'light' : 'dark'}
    </button>
  </header>
)

export default Header
