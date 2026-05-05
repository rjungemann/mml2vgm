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
  
  // Base path for Cloudflare Pages (empty for root deployment)
  base: process.env.BASE_PATH || '/',
  
  // Resolve path aliases
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@/components': path.resolve(__dirname, './src/components'),
      '@/services': path.resolve(__dirname, './src/services'),
      '@/stores': path.resolve(__dirname, './src/stores'),
      '@/types': path.resolve(__dirname, './src/types'),
      '@/utils': path.resolve(__dirname, './src/utils'),
      // Alias for the WASM package
      'mml2vgm-wasm': path.resolve(__dirname, '../mml2vgm-wasm/pkg'),
    },
  },
  
  // WASM-specific settings
  build: {
    target: 'es2022',
    // Ensure WASM files are copied to dist
    assetsInlineLimit: 0,
  },
  
  // Server settings for development
  server: {
    headers: {
      // Required for SharedArrayBuffer in Chrome/Edge
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
  
  // Optimize dependencies
  optimizeDeps: {
    // Exclude WASM module from pre-bundling
    exclude: ['mml2vgm-wasm'],
  },
  
  // Worker configuration - Vite 5 uses a function for plugins
  worker: {
    format: 'es',
    plugins: () => [wasm()],
    // Ensure workers inherit the same resolve aliases as the main bundle
    resolve: {
      alias: {
        '@': path.resolve(__dirname, './src'),
        '@/components': path.resolve(__dirname, './src/components'),
        '@/services': path.resolve(__dirname, './src/services'),
        '@/stores': path.resolve(__dirname, './src/stores'),
        '@/types': path.resolve(__dirname, './src/types'),
        '@/utils': path.resolve(__dirname, './src/utils'),
        'mml2vgm-wasm': path.resolve(__dirname, '../mml2vgm-wasm/pkg'),
      },
    },
  },
});
