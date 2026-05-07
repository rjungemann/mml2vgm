# WASM Compilation Hang: Debugging Strategy Guide

> **Historical document — issue resolved May 2026.**
> The hang was caused by `chars().nth(position)` in the lexer (O(n²) per character).
> Fixed by switching to `source[position..].chars().next()`.
> See [Performance_Improvement_Plan.md](./Performance_Improvement_Plan.md) for details.
> This document is kept for reference in case similar regressions are introduced.

## Problem Statement

The browser IDE's WASM compilation function (`compile_mml()`) hangs indefinitely when processing real MML files (403+ bytes with comments and structure), despite completing quickly on simple test cases (14-byte MML in 7ms).

**Key Evidence:**
- Browser logs show: `[WasmWrapper] Calling compile_mml at 2026-05-05T19:09:30.749Z`
- Function never returns - no result, no error, no timeout
- Occurs on both Chrome and Firefox
- CLI compilation works fine (proven by integration tests passing)
- Simple smoke test MML (`o4 c4 d4 e4 f4`) compiles in 7ms
- Real test MML files hang indefinitely

**Hypothesis:** The hang is in the Rust lexer, parser, or code generation logic - not in the JavaScript/Worker integration layer.

---

## Root Cause Categories

### 1. Lexer Infinite Loop (MOST LIKELY)

**Why it's likely:**
- Lexer is the first processing step - failures here block everything downstream
- Comment handling involves complex state transitions
- Recent optimizations changed character iteration logic
- Multi-line constructs can create edge cases

**Areas to investigate:**
- **Comment parsing:** Comments may not terminate correctly, causing infinite loop in `next_token()`
- **Newline handling:** The recursive `skip_whitespace()` and newline backtracking logic could create loops
- **String boundaries:** UTF-8 slicing (`self.source[pos..].chars()`) could misalign on multi-byte characters
- **Position tracking:** Mismatch between byte position and character position in UTF-8 text

**Test approach:**
```
1. Add debug logging at lexer entry/exit
2. Log token count every 100 tokens
3. Watch for patterns in token stream
4. Test with progressively longer comments
5. Test with various UTF-8 characters
```

### 2. Parser Infinite Recursion or Cyclic Grammar

**Why it's possible:**
- Complex MML grammar with nested structures
- Recent parser context optimization changed flow
- Certain input patterns could trigger left recursion or mutual recursion

**Areas to investigate:**
- **Recursive descent patterns:** Check for functions that call themselves without consuming input
- **The `definition_context` flag:** Boolean flag may not correctly track context boundaries
- **Token consumption:** Verify parser always advances on every iteration
- **Grammar ambiguities:** Certain token sequences might match multiple grammar rules

**Test approach:**
```
1. Add recursion depth counter to parser
2. Log parser stack depth every 1000 reductions
3. Test with increasing nesting levels
4. Check for unconsumed token loops
5. Validate state machine transitions
```

### 3. Codegen Infinite Loop or Allocation Explosion

**Why it's possible:**
- Code generation could enter exponential loops with certain patterns
- Memory pressure could cause slowdown masquerading as hang
- VGM byte emission could have unexpected cycles

**Areas to investigate:**
- **Note expansion loops:** Patterns that expand to many VGM commands
- **Macro expansion:** Recursive macro definitions
- **Memory allocations:** Vec/String growing unbounded

**Test approach:**
```
1. Monitor memory usage during compilation
2. Log VGM command emission rate
3. Test with maximally-nested structures
4. Profile allocations with perf/valgrind
```

### 4. Deadlock or Blocking Operation

**Why it's unlikely but possible:**
- WASM is single-threaded, so deadlock is less likely
- Could be blocking I/O (file reads) in code generation
- Potential synchronous waiting on external resource

**Areas to investigate:**
- **File I/O:** Any `std::fs::*` calls in the compilation path
- **Synchronous operations:** Any operations that might block the event loop
- **External dependencies:** Check dependencies for blocking behavior

---

## Debugging Approaches

### Approach 1: Binary Search with Test Cases

**Goal:** Isolate the problematic MML pattern

**Steps:**
1. Start with the real 403-byte test MML file that hangs
2. Bisect the file content - test first half, second half
3. Find the minimum input that triggers hang
4. Identify the specific MML construct that causes it

**Implementation:**
```bash
# Create progressively smaller test files
head -c 200 test.mml > test_half.mml
head -c 100 test.mml > test_quarter.mml
# etc.
```

**Expected outcome:** Identify the exact MML pattern that breaks the compiler

### Approach 2: Instrumentation with Logging

**Goal:** Trace execution flow and identify where hang occurs

**Rust code changes needed:**

1. **Lexer logging (mml2vgm-rs/src/compiler/lexer.rs):**
```rust
pub fn next_token(&mut self) -> Token {
    let start_pos = self.position;
    let token = self._next_token_impl();
    eprintln!("[Lexer] Token {} at pos {}: {:?}", self.token_count, start_pos, token);
    self.token_count += 1;
    
    if self.token_count % 100 == 0 {
        eprintln!("[Lexer] Processed {} tokens, pos {}/{}", 
                  self.token_count, self.position, self.source.len());
    }
    
    token
}
```

2. **Parser logging (mml2vgm-rs/src/compiler/parser.rs):**
```rust
fn parse(&mut self) -> Result<Module, Vec<CompileError>> {
    eprintln!("[Parser] Starting parse, {} tokens available", self.tokens.len());
    // ... existing code ...
    // Log at significant milestones:
    eprintln!("[Parser] Parsed {} parts", module.parts.len());
}
```

3. **Codegen logging (mml2vgm-rs/src/codegen/mod.rs):**
```rust
pub fn emit(&mut self) -> Result<Vec<u8>, Vec<CompileError>> {
    eprintln!("[Codegen] Starting emission");
    // ... existing code ...
    eprintln!("[Codegen] Emitted {} VGM bytes", self.output.len());
}
```

**Build and test:**
```bash
just wasm-build  # Rebuild with logging
# Browser console will show logs
```

### Approach 3: WASM Profiling with Browser DevTools

**Goal:** Identify CPU hotspots

**Steps:**
1. Open Chrome DevTools Performance tab
2. Record performance during compilation
3. Look for long functions that don't return
4. Check call stack when recording ends

**Expected findings:**
- Function name that consumes all CPU time
- Call stack showing which Rust functions are looping

### Approach 4: WASM Linear Memory Inspection

**Goal:** Detect stack overflow or memory corruption

**JavaScript code (in wasmWrapper.ts):**
```typescript
export function compileMmlWithMemoryMonitoring(mml: string, optionsJson: string): any {
  if (!wasmModule) {
    throw new Error('WASM module not initialized');
  }

  const memory = wasmModule.memory;
  const initialPages = memory.buffer.byteLength / 65536;
  
  console.log(`[Profiling] Initial memory: ${initialPages} pages (${memory.buffer.byteLength} bytes)`);

  // Monitor memory every 100ms during compilation
  const monitorInterval = setInterval(() => {
    const currentPages = memory.buffer.byteLength / 65536;
    console.log(`[Profiling] Memory: ${currentPages} pages`);
  }, 100);

  try {
    const startTime = performance.now();
    const result = wasmModule.compile_mml(mml, optionsJson);
    const duration = performance.now() - startTime;
    
    clearInterval(monitorInterval);
    console.log(`[Profiling] Completed in ${duration}ms`);
    return result;
  } catch (error) {
    clearInterval(monitorInterval);
    throw error;
  }
}
```

### Approach 5: Test with Reduced Input

**Goal:** Eliminate variables until problem surfaces

**Test sequence:**
1. Simple note: `c4`
2. Multiple notes: `c4 d4 e4 f4` (known working)
3. With octave: `o4 c d e f`
4. With multiple octaves: `o3 c o4 d o5 e`
5. With comments: `c4 ; comment`
6. Multi-line: `c4\nd4\ne4`
7. Real file structure (incrementally add back features)

**Testing code:**
```typescript
// In smoke-test.mjs or dedicated test script
const testCases = [
  { name: 'single-note', mml: 'c4' },
  { name: 'scale', mml: 'c4 d4 e4 f4' },
  { name: 'octave-change', mml: 'o4 c o5 d o6 e' },
  { name: 'with-comment', mml: 'c4 ; test' },
  { name: 'multiline', mml: 'c4\nd4\ne4' },
];

for (const testCase of testCases) {
  console.log(`Testing: ${testCase.name}`);
  try {
    const result = await compileWithTimeout(testCase.mml, 5000);
    console.log(`✓ ${testCase.name} passed`);
  } catch (error) {
    console.log(`✗ ${testCase.name} hung or errored: ${error.message}`);
    break;
  }
}
```

---

## Specific Code Areas to Investigate

### 1. Lexer Comment Handling
**File:** `mml2vgm-rs/src/compiler/lexer.rs`

Look for:
- The `skip_comment()` function - verify it terminates
- Newline handling in comments - check for infinite loops
- Interaction between `skip_whitespace()` and `skip_comment()`

### 2. Lexer Newline Logic
**File:** `mml2vgm-rs/src/compiler/lexer.rs`

The recursive `skip_whitespace()` with newline backtracking could be problematic:
```rust
fn skip_whitespace(&mut self) {
    while self.is_whitespace(self.current_char()) {
        if self.current_char() == '\n' {
            self.newlines.push(self.position);
            // Check for potential infinite loop here
        }
        self.advance();
    }
}
```

### 3. Parser Token Consumption
**File:** `mml2vgm-rs/src/compiler/parser.rs`

Verify every parsing function advances the token position:
- Look for `while` loops that don't call `self.advance()`
- Check for functions that might match the same token repeatedly
- Validate the `definition_context` flag prevents backtracking

### 4. Codegen Note Expansion
**File:** `mml2vgm-rs/src/codegen/` (multiple files)

Look for:
- Note/rest expansion logic - ensure it terminates
- Macro expansion - check for recursive definitions
- Effect processing - verify loop termination

---

## Testing Strategy

### Quick Wins (Test First)
1. **Test with comments removed:** Does the real MML hang without comments?
2. **Test with single line:** Does it hang if comments are all removed?
3. **Test with shortened values:** Does shortening note lengths help?

### Systematic Testing
1. **Create test harness** that accepts MML string and timeout
2. **Run binary search** to find minimal problematic input
3. **Create regression test** once minimal case is found
4. **Document the pattern** for future reference

### Integration Test Enhancement
Add a dedicated test in the test suite:
```rust
#[cfg(test)]
mod wasm_hang_tests {
    #[test]
    fn test_real_mml_file_compiles() {
        let mml = include_str!("../../browser-ide/public/samples/general_test.gwi");
        let options = CompileOptions::default();
        
        // Should complete within 5 seconds
        let start = Instant::now();
        let result = compile_mml(mml, options);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Compilation should succeed");
        assert!(duration.as_secs() < 5, "Should compile quickly, took {:?}", duration);
    }
}
```

---

## Diagnostic Output Format

When adding logging, use consistent format:

```
[Component] Action: details
[Lexer] Token 1234 at pos 5678: Identifier("c")
[Parser] Parsed 5 parts, in_definition=false
[Codegen] Emitted 2048 VGM bytes
```

This makes it easy to filter logs and track flow through components.

---

## Next Steps (Priority Order)

1. **Quick test:** Remove all comments from test MML, try compiling - proves if comments are the issue
2. **Add logging:** Instrument lexer's `next_token()` and parser's main loop
3. **Run with test MML:** Browser test will show where logs stop
4. **Binary search:** Cut down test case to minimal problematic input
5. **Profile:** Use browser DevTools to identify CPU hotspot
6. **Fix:** Once root cause identified, implement fix
7. **Prevent regression:** Add test case to test suite

---

## Success Criteria

- Compilation of real test MML completes in < 2 seconds
- No infinite loops detected in logs
- Memory usage stays constant (no explosion)
- CPU usage shows steady progress, not stuck in one function
- Regression test prevents hang from returning

