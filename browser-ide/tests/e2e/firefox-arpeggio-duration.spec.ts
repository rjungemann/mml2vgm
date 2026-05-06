import { expect, test } from '@playwright/test';

function parseMetric(text: string, key: string): number {
  const line = text
    .split('\n')
    .map((s) => s.trim())
    .find((s) => s.toLowerCase().startsWith(key.toLowerCase()));
  if (!line) return NaN;
  const value = line.slice(key.length).trim();
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : NaN;
}

test('arpeggio playback remains audible beyond initial burst in Firefox', async ({ page }) => {
  await page.goto('/');

  // Wait for IDE to initialize (Menu bar shown means WASM init completed).
  await expect(page.getByRole('menubar', { name: 'Main menu' })).toBeVisible();

  // Load sample: Examples -> Arpeggio.
  await page.getByRole('button', { name: 'Examples' }).click();
  await page.getByRole('menuitem', { name: 'Arpeggio' }).click();

  // Compile+play from Play menu.
  await page.getByRole('button', { name: 'Play' }).click();
  await page.getByRole('menuitem', { name: /Play \(F5\)/ }).click();

  // Open runtime debug panel and read metrics.
  const runtimeDebugButton = page.getByRole('button', { name: /^Runtime Debug/ }).first();
  await runtimeDebugButton.click();

  const runtimePanel = page.locator('div', { hasText: 'Parsed VGM commands:' }).first();
  await expect(runtimePanel).toBeVisible();

  // Capture metrics shortly after start and after >1 second.
  await page.waitForTimeout(200);
  const earlyText = await page.locator('body').innerText();
  const earlyBuffers = parseMetric(earlyText, 'Buffers generated:');

  await page.waitForTimeout(1200);
  const laterText = await page.locator('body').innerText();
  const laterBuffers = parseMetric(laterText, 'Buffers generated:');
  const lastPeak = parseMetric(laterText, 'Last peak:');
  const silentStreak = parseMetric(laterText, 'Silent buffer streak:');

  expect(Number.isFinite(earlyBuffers)).toBe(true);
  expect(Number.isFinite(laterBuffers)).toBe(true);
  expect(laterBuffers).toBeGreaterThan(earlyBuffers);

  // Regression guard: playback should remain active/audible, not decay to silence immediately.
  expect(lastPeak).toBeGreaterThan(0);
  expect(silentStreak).toBe(0);

  await expect(page.getByText('Audio playing: yes')).toBeVisible();
});
