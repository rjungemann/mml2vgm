import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import path from 'path';

export default defineConfig({
    plugins: [
        react(),
        wasm(),
    ],
    
    test: {
        // Environment
        environment: 'jsdom',
        globals: true,
        setupFiles: ['./src/test/setup.ts'],
        
        // Coverage
        coverage: {
            provider: 'v8',
            reporter: ['text', 'json', 'html'],
            reportsDirectory: './coverage',
            exclude: [
                'node_modules/',
                'dist/',
                'public/',
                '**/*.d.ts',
                '**/*.config.*',
                '.eslintrc.cjs',
            ],
        },
        
        // Test files
        include: ['src/**/*.{test,spec}.{js,ts,jsx,tsx}'],
        exclude: [
            'node_modules/',
            'dist/',
        ],
        
        // Watch mode
        watch: false,
        
        // Pass through environment variables
        env: {
            NODE_ENV: 'test',
        },
        
        // Mock WASM for tests
        // Note: In test environment, we mock the WASM module
        // CSS modules
        css: true,
    },
    
    // Resolve aliases
    resolve: {
        alias: {
            '@': path.resolve(__dirname, './src'),
        },
    },
});
