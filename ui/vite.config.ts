import { defineConfig } from 'vite'
import solid from 'vite-plugin-solid'

export default defineConfig({
  plugins: [solid()],
  build: {
    outDir: '../dist/ui',
    emptyOutDir: true
  },
  server: {
    proxy: {
      '/api': 'http://localhost:8080',
      '/files': 'http://localhost:8080',
      '/r4': 'http://localhost:8080',
      '/r4b': 'http://localhost:8080',
      '/r5': 'http://localhost:8080',
      '/r6': 'http://localhost:8080'
    }
  }
})