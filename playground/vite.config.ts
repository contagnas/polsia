import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import wasm from 'vite-plugin-wasm'
import topLevelAwait from 'vite-plugin-top-level-await'
import tailwindcss from '@tailwindcss/vite'

const watchPlsFiles = {
  name: 'watch-text-files',
  // watch ANY .txt in /data or wherever
  handleHotUpdate({ file, server }) {
    if (file.endsWith('.pls')) {
      // tell Vite “please do a full reload”
      server.ws.send({ type: 'full-reload', path: '*' })
    }
  }
}

// https://vite.dev/config/
export default defineConfig({
  base: '/polsia/',
  plugins: [react(), wasm(), topLevelAwait(), tailwindcss(), watchPlsFiles],
})
