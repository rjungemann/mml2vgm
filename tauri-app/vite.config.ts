import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import path from 'path';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react(),
    wasm(),
  ],
  
  // Base path for Tauri - use relative paths
  base: './',
  
  // Resolve path aliases - point to browser-ide
  resolve: {
    alias: {
      '@': path.resolve(__dirname, '../browser-ide/src'),
      '@/components': path.resolve(__dirname, '../browser-ide/src/components'),
      '@/services': path.resolve(__dirname, '../browser-ide/src/services'),
      '@/stores': path.resolve(__dirname, '../browser-ide/src/stores'),
      '@/types': path.resolve(__dirname, '../browser-ide/src/types'),
      '@/utils': path.resolve(__dirname, '../browser-ide/src/utils'),
      'mml2vgm-wasm': path.resolve(__dirname, '../mml2vgm-wasm/pkg'),
    },
  },
  
  build: {
    target: 'es2022',
    outDir: 'dist',
    assetsInlineLimit: 0,
    emptyOutDir: true,
    rollupOptions: {
      input: path.resolve(__dirname, 'index.html'),
    },
  },
  
  server: {
    port: 5173,
    strictPort: true,
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
  
  optimizeDeps: {
    exclude: ['mml2vgm-wasm'],
  },
  
  worker: {
    format: 'es',
  },
});
