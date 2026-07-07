import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './design/tokens/colors.css'
import './design/tokens/spacing.css'
import './design/tokens/typography.css'
import './design/tokens/elevation.css'
import './design/styles.css'
import './base.css'
import App from './App.tsx'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
