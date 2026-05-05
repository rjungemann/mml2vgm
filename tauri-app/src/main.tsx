/**
 * Tauri Desktop App Main Entry Point
 * 
 * This loads the browser-ide App component and provides Tauri integration.
 */

import React from 'react';
import ReactDOM from 'react-dom/client';
import App from '../browser-ide/src/App';
import './index.css';

// Tauri desktop-specific functionality
import { invoke } from '@tauri-apps/api/core';

// Make Tauri APIs available globally for the browser-ide
window.invoke = invoke;

declare global {
  interface Window {
    invoke: typeof invoke;
  }
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.Suspense fallback={<div>Loading mml2vgm IDE...</div>}>
    <App />
  </React.Suspense>
);
