#!/usr/bin/env node

/**
 * Smoke Test: Browser IDE Compilation
 *
 * This test verifies that the browser IDE can compile MML files without hanging.
 * It uses Playwright to open the browser IDE and attempt a compilation,
 * then verifies that the result is returned within a reasonable time.
 */

import { chromium } from 'playwright';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));

const TEST_TIMEOUT = 30000; // 30 seconds total timeout
const COMPILE_TIMEOUT = 15000; // 15 seconds for compilation
const BROWSER_URL = 'http://localhost:5173';

// Simple MML that should compile quickly
const SIMPLE_MML = `o4 c4 d4 e4 f4`;

async function runSmokeTest() {
  console.log('🧪 Starting Browser IDE Smoke Test');
  console.log(`   Browser URL: ${BROWSER_URL}`);
  console.log(`   Test timeout: ${TEST_TIMEOUT}ms`);
  console.log(`   Compile timeout: ${COMPILE_TIMEOUT}ms`);
  console.log('');

  let browser;
  let page;

  try {
    // Launch browser
    console.log('📱 Launching browser...');
    browser = await chromium.launch({ headless: true });
    page = await browser.newPage();

    // Navigate to browser IDE
    console.log(`🌐 Navigating to ${BROWSER_URL}...`);
    await page.goto(BROWSER_URL, { waitUntil: 'networkidle', timeout: 10000 });
    console.log('✓ Page loaded');

    // Wait for the IDE to be ready
    console.log('⏳ Waiting for IDE to initialize...');
    await page.waitForFunction(() => {
      const appRoot = document.querySelector('#root');
      return appRoot && appRoot.textContent.includes('Compile');
    }, { timeout: 10000 });
    console.log('✓ IDE initialized');

    // Set up console log capture
    const consoleLogs = [];
    page.on('console', msg => {
      const text = msg.text();
      consoleLogs.push(text);
      // Log all messages with [Worker], [compileStore], etc.
      if (text.includes('[Worker]') || text.includes('[compileStore]') ||
          text.includes('[WasmWrapper]') || text.includes('[WorkerManager]')) {
        console.log(`   📝 ${text}`);
      }
    });

    // Also capture errors
    page.on('pageerror', error => {
      console.log(`   ❌ Page error: ${error.message}`);
      consoleLogs.push(`PAGE_ERROR: ${error.message}`);
    });

    // Insert MML into editor
    console.log('✏️  Inserting MML into editor...');
    const editorSet = await page.evaluate((mml) => {
      // Try to find the Monaco editor or textarea
      const monacoEditor = document.querySelector('.monaco-editor');
      const textarea = document.querySelector('textarea');
      const contentEditable = document.querySelector('[contenteditable="true"]');

      if (monacoEditor && window.monaco?.editor?.getEditors) {
        const editorInstance = window.monaco.editor.getEditors()[0];
        if (editorInstance) {
          editorInstance.setValue(mml);
          return { type: 'monaco', success: true, value: editorInstance.getValue().substring(0, 50) };
        }
      }

      if (textarea) {
        textarea.value = mml;
        textarea.dispatchEvent(new Event('input', { bubbles: true }));
        return { type: 'textarea', success: true, value: textarea.value.substring(0, 50) };
      }

      if (contentEditable) {
        contentEditable.textContent = mml;
        contentEditable.dispatchEvent(new Event('input', { bubbles: true }));
        return { type: 'contenteditable', success: true, value: contentEditable.textContent.substring(0, 50) };
      }

      return { type: 'none', success: false };
    }, SIMPLE_MML);

    console.log(`   Result: ${JSON.stringify(editorSet)}`);

    if (!editorSet.success) {
      console.log('⚠️  Could not set editor content');
      throw new Error('Failed to set editor content');
    }

    await new Promise(r => setTimeout(r, 1000));

    // Click compile button
    console.log('🔘 Clicking Compile button...');
    const buttons = await page.$$('button');
    console.log(`   Found ${buttons.length} buttons`);

    // Find all Compile buttons and their info
    const allCompileButtons = [];
    for (let i = 0; i < buttons.length; i++) {
      const text = await buttons[i].textContent();
      if (text.includes('Compile')) {
        const buttonInfo = await page.evaluate((idx) => {
          const btn = document.querySelectorAll('button')[idx];
          return {
            text: btn.textContent,
            id: btn.id,
            className: btn.className,
            onclick: !!btn.onclick,
            dataTestid: btn.getAttribute('data-testid'),
            visible: btn.offsetParent !== null,
          };
        }, i);
        allCompileButtons.push({ index: i, info: buttonInfo });
        console.log(`   Button ${i}: ${JSON.stringify(buttonInfo)}`);
      }
    }

    // Click the first visible compile button that's not a menu item
    let found = false;
    for (const { index, info } of allCompileButtons) {
      if (info.visible && !info.className.includes('menu-item') && info.onclick) {
        console.log(`   ✓ Found real Compile button at index ${index}`);

        // Click the button (use correct index)
        await buttons[index].click();
        console.log(`   ✓ Click sent`);

        // Verify click was processed by checking for Stop button
        const stopFound = await page.evaluate(() => {
          return Array.from(document.querySelectorAll('button'))
            .some(btn => btn.textContent.includes('Stop'));
        });
        console.log(`   Stop button appeared: ${stopFound}`);

        // Check state after click
        await new Promise(r => setTimeout(r, 500));
        const afterState = await page.evaluate(() => {
          const status = Array.from(document.querySelectorAll('*'))
            .find(el => el.textContent.includes('compiling') || el.textContent.includes('100%'));
          return {
            statusElement: status ? 'found' : 'not found',
            progressText: document.body.innerText.substring(0, 100),
          };
        }).catch(() => ({}));
        console.log(`   After click state: ${JSON.stringify(afterState)}`);

        found = true;
        break;
      }
    }

    if (!found) {
      console.log('⚠️  Could not find Compile button');
      throw new Error('Compile button not found');
    }

    console.log('📊 Monitoring compilation...');

    // Wait for compilation to complete or timeout
    let compiled = false;
    let errorOccurred = false;
    let errorMessage = '';

    const startTime = Date.now();
    let sawStopButton = false;

    // Monitor the button text - when it changes from "Stop Compilation" back to "Compile", it's done
    while (Date.now() - startTime < COMPILE_TIMEOUT) {
      const status = await page.evaluate(() => {
        // Look for any element that indicates success or error
        const successEl = document.body.innerText.toLowerCase().includes('successfully');
        const errorEl = document.body.innerText.toLowerCase().includes('error:');
        const allBtns = Array.from(document.querySelectorAll('button'));
        const hasStopButton = allBtns.some(btn => btn.textContent.includes('Stop'));
        const hasCompileButton = allBtns.some(btn => btn.textContent.includes('Compile'));

        return {
          hasSuccess: successEl,
          hasError: errorEl,
          hasStopButton: hasStopButton,
          hasCompileButton: hasCompileButton,
          buttonTexts: allBtns.map(b => b.textContent).filter(t => t.includes('Compile') || t.includes('Stop')),
          allText: document.body.innerText.substring(0, 300),
        };
      });

      console.log(`   Stop: ${status.hasStopButton}, Compile: ${status.hasCompileButton}, Success: ${status.hasSuccess}, Error: ${status.hasError}`);
      if (status.buttonTexts.length > 0) {
        console.log(`   Buttons: ${status.buttonTexts.join(' | ')}`);
      }

      if (status.hasStopButton) {
        sawStopButton = true;
      }

      if (status.hasSuccess) {
        compiled = true;
        console.log('✓ Compilation SUCCEEDED (success message found)');
        break;
      }

      if (status.hasError) {
        errorOccurred = true;
        const errorMatch = status.allText.match(/Error:(.+?)(?:\n|$)/);
        errorMessage = errorMatch ? errorMatch[1] : 'Unknown error';
        console.log(`✗ Compilation ERROR: ${errorMessage}`);
        break;
      }

      // If we saw Stop button before, and now it's gone, compilation finished
      if (sawStopButton && !status.hasStopButton && status.hasCompileButton) {
        console.log('✓ Compilation completed (Stop button disappeared)');
        compiled = true;
        break;
      }

      await new Promise(r => setTimeout(r, 1000));
    }

    const elapsedMs = Date.now() - startTime;

    console.log('');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    if (compiled) {
      console.log(`✅ SMOKE TEST PASSED`);
      console.log(`   Compilation completed in ${elapsedMs}ms`);
      console.log(`   MML length: ${SIMPLE_MML.length} chars`);
      return true;
    } else if (errorOccurred) {
      console.log(`❌ SMOKE TEST FAILED - Compilation error`);
      console.log(`   Error: ${errorMessage}`);
      return false;
    } else if (elapsedMs >= COMPILE_TIMEOUT) {
      console.log(`❌ SMOKE TEST FAILED - COMPILATION TIMEOUT`);
      console.log(`   No result after ${elapsedMs}ms`);
      console.log(`   Total console logs captured: ${consoleLogs.length}`);
      if (consoleLogs.length > 0) {
        console.log('   Sample logs:');
        consoleLogs.slice(-10).forEach(log => console.log(`   ${log.substring(0, 100)}`));
      }
      return false;
    }

  } catch (error) {
    console.log(`❌ SMOKE TEST FAILED - Exception`);
    console.log(`   Error: ${error.message}`);
    console.log(`   Stack: ${error.stack}`);
    return false;
  } finally {
    if (page) await page.close();
    if (browser) await browser.close();
  }
}

// Run the test
runSmokeTest().then(passed => {
  process.exit(passed ? 0 : 1);
}).catch(err => {
  console.error('Fatal error:', err);
  process.exit(1);
});
