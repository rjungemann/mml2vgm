# Browser IDE Smoke Tests

This directory contains smoke tests for the mml2vgm Browser IDE.

## Test Files

- `smoke.test.html` - Interactive HTML page with manual smoke tests

## Running Tests

### Option 1: Interactive Browser Testing

1. Start the dev server:
   ```bash
   cd browser-ide
   npm run dev
   ```

2. Open `http://localhost:5173` in your browser
3. Verify the app loads without errors
4. Open `tests/smoke.test.html` in your browser
5. Click each test button to run the tests

### Option 2: Automated Testing with Puppeteer

Install puppeteer:
```bash
npm install --save-dev puppeteer
```

Run the smoke test script:
```bash
node tests/smoke.test.js
```

### Option 3: cURL Tests

For basic HTTP testing without a browser:

```bash
# Start the dev server
npm run dev

# Test page loads
curl -s http://localhost:5173 | grep -q "mml2vgm Browser IDE" && echo "PASS" || echo "FAIL"

# Test static assets
curl -s -o /dev/null -w "%{http_code}" http://localhost:5173/src/main.tsx
```

## Test Cases

### Core Functionality Tests

1. **Page Load** - Verify the page loads without JavaScript errors
2. **Monaco Editor Initialization** - Verify Monaco Editor loads from CDN
3. **WASM Module Load** - Verify WASM module can be imported
4. **WASM Compilation** - Verify MML can be compiled to VGM
5. **WASM Tokenization** - Verify MML can be tokenized for syntax highlighting
6. **Settings Load** - Verify settings are loaded correctly
7. **Document Creation** - Verify documents can be created and managed

### Integration Tests

8. **Compilation Flow** - Test full flow: Editor → compileStore → wasmService → WASM
9. **Audio Playback** - Test chip player initialization and sample generation
10. **File Operations** - Test file open/save functionality

### UI Tests

11. **Panel Rendering** - Verify all panels render correctly
12. **Theme Switching** - Test dark/light theme switching
13. **Syntax Highlighting** - Verify MML syntax highlighting works
14. **Error Display** - Test error list panel with compilation errors

## Test Results

| Test | Description | Expected | Actual | Status |
|------|-------------|----------|--------|--------|
| T1 | Page Load | No errors | | ⏳ |
| T2 | Monaco Editor | Loads from CDN | | ⏳ |
| T3 | WASM Module Load | Import succeeds | | ⏳ |
| T4 | WASM Compilation | Returns VGM data | | ⏳ |
| T5 | WASM Tokenization | Returns tokens | | ⏳ |
| T6 | Settings Load | Loads defaults | | ⏳ |
| T7 | Document Creation | Creates doc | | ⏳ |

## Browser Compatibility

| Browser | WASM | AudioWorklet | Web MIDI | File System API | Status |
|---------|------|--------------|----------|-----------------|--------|
| Chrome 120+ | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes | Full |
| Firefox 120+ | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes | Full |
| Safari 17+ | ✅ Yes | ❌ No | ❌ No | ❌ No | Partial |
| Edge 120+ | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes | Full |

## Known Limitations

1. **Safari**: AudioWorklet not supported, will use ScriptProcessorNode fallback
2. **Safari**: Web MIDI API not supported
3. **Safari**: File System Access API not supported
4. **All**: WASM module is ~318KB, may take time to load on slow connections

## Performance Benchmarks

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| WASM Load Time | < 1s | | ⏳ |
| Compilation Time (simple MML) | < 100ms | | ⏳ |
| Tokenization Time | < 50ms | | ⏳ |
| Audio Latency | < 50ms | | ⏳ |

## Continuous Integration

For CI testing, add to your workflow:

```yaml
- name: Run Smoke Tests
  run: |
    npm run build
    npm run dev &
    sleep 10
    curl -s http://localhost:5173 | grep -q "mml2vgm Browser IDE"
```

## Writing New Tests

1. Add test to `smoke.test.html`
2. Follow the existing pattern:
   - Test function with `test*` prefix
   - Call `setResult()` with test ID, pass/fail, and message
   - Use `log()` for console output
3. Update the test table in this README

## Debugging

If tests fail:

1. Check browser console for errors
2. Verify dev server is running: `npm run dev`
3. Check WASM module path in `vite.config.ts`
4. Verify Monaco CDN is accessible
5. Check TypeScript compilation: `npm run check`
6. Check build: `npm run build`
