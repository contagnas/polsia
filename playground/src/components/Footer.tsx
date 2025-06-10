import Marquee from 'react-fast-marquee'

interface Props {
  error: boolean
}

const Footer = ({ error }: Props) => {
  const color = error ? 'text-keyword' : 'text-accent'
  return (
    <footer className={`text-xs border-t border-variable bg-panel ${color}`}>
      <Marquee autoFill direction={error ? 'right' : 'left'}>
        {error ? 'AISLOP ' : 'POLSIA '} &nbsp;
      </Marquee>
    </footer>
  )
}

export default Footer
