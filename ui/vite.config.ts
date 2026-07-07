import { writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  // Tauri's devUrl is fixed to http://localhost:5173; fail fast if taken.
  server: { port: 5173, strictPort: true },
  plugins: [
    react(),
    {
      // Vite empties dist/ on build; restore the tracked keep-file that
      // Tauri codegen needs (frontendDist must exist at compile time).
      name: 'restore-dist-gitkeep',
      closeBundle() {
        writeFileSync(join(import.meta.dirname, 'dist', '.gitkeep'), '')
      },
    },
  ],
})
