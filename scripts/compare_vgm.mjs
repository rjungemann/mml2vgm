#!/usr/bin/env node
/**
 * compare_vgm.mjs — Golden Master VGM command sequence comparator
 *
 * Parses two directories of .vgm files, extracts chip command sequences,
 * and reports differences. Header fields (timestamps, version, etc.) are
 * intentionally ignored — only the chip register writes and timing matter.
 *
 * Usage:
 *   node scripts/compare_vgm.mjs <reference-dir> <current-dir>
 */

import fs from 'node:fs';
import path from 'node:path';

function usage() {
  console.error('Usage: node scripts/compare_vgm.mjs <reference-dir> <current-dir>');
  process.exit(1);
}

/**
 * Parse a VGM buffer and extract the command sequence as a list of
 * [cmd, ...args] tuples. Returns { commands, dataBlocks }.
 * dataBlocks: Array of { type, data }
 */
function parseVgm(buf, filePath) {
  if (buf.length < 64) throw new Error(`VGM too small: ${filePath}`);
  if (buf.toString('ascii', 0, 4) !== 'Vgm ') throw new Error(`Not a VGM file: ${filePath}`);

  const version = buf.readUInt32LE(8);
  const eofOffset = buf.readUInt32LE(4);
  const dataEnd = Math.min(eofOffset + 4, buf.length);

  // VGM data offset field at 0x34 (VGM >= 1.50)
  let offset = 0x40;
  if (version >= 0x150 && buf.length >= 0x38) {
    const vgmDataOffset = buf.readUInt32LE(0x34);
    if (vgmDataOffset > 0) {
      offset = 0x34 + vgmDataOffset;
    }
  }

  const commands = [];
  const dataBlocks = [];

  while (offset < dataEnd && offset < buf.length) {
    const cmd = buf[offset++];

    switch (cmd) {
      case 0x66: // end of data
        return { commands, dataBlocks };

      case 0x50: { // SN76489
        if (offset + 1 > buf.length) break;
        commands.push([cmd, buf[offset++]]);
        break;
      }
      case 0x51: case 0x52: case 0x53: case 0x54:
      case 0x55: case 0x56: case 0x57: case 0x5A:
      case 0x5B: case 0x5C: case 0x5D: case 0x5E: case 0x5F: {
        if (offset + 2 > buf.length) break;
        commands.push([cmd, buf[offset], buf[offset + 1]]);
        offset += 2;
        break;
      }
      case 0x58: case 0x59: { // SegaPCM
        if (offset + 3 > buf.length) break;
        commands.push([cmd, buf[offset], buf[offset + 1], buf[offset + 2]]);
        offset += 3;
        break;
      }
      case 0x61: { // wait N samples
        if (offset + 2 > buf.length) break;
        const samples = buf.readUInt16LE(offset);
        offset += 2;
        commands.push([cmd, samples]);
        break;
      }
      case 0x62: // wait 735
        commands.push([cmd]);
        break;
      case 0x63: // wait 882
        commands.push([cmd]);
        break;
      case 0x67: { // data block
        if (offset + 6 > buf.length) break;
        offset++; // compat 0x66
        const blockType = buf[offset++];
        const blockSize = buf.readUInt32LE(offset);
        offset += 4;
        const end = Math.min(offset + blockSize, buf.length);
        const data = buf.slice(offset, end);
        dataBlocks.push({ type: blockType, data });
        offset = end;
        break;
      }
      case 0x70: case 0x71: case 0x72: case 0x73:
      case 0x74: case 0x75: case 0x76: case 0x77:
      case 0x78: case 0x79: case 0x7A: case 0x7B:
      case 0x7C: case 0x7D: case 0x7E: case 0x7F: {
        // wait n+1 samples (0x7n = wait n+1)
        commands.push([cmd]);
        break;
      }
      case 0x80: case 0x81: case 0x82: case 0x83:
      case 0x84: case 0x85: case 0x86: case 0x87:
      case 0x88: case 0x89: case 0x8A: case 0x8B:
      case 0x8C: case 0x8D: case 0x8E: case 0x8F: {
        // YM2612 PCM write + wait
        commands.push([cmd]);
        break;
      }
      case 0xA0: { // AY8910
        if (offset + 2 > buf.length) break;
        commands.push([cmd, buf[offset], buf[offset + 1]]);
        offset += 2;
        break;
      }
      case 0xC0: case 0xC1: { // RF5C68 / RF5C164
        if (offset + 3 > buf.length) break;
        commands.push([cmd, buf[offset], buf[offset + 1], buf[offset + 2]]);
        offset += 3;
        break;
      }
      case 0xE0: { // seek in PCM data bank
        if (offset + 4 > buf.length) break;
        commands.push([cmd, buf.readUInt32LE(offset)]);
        offset += 4;
        break;
      }
      default:
        // Unknown command — skip (best effort)
        break;
    }
  }
  return { commands, dataBlocks };
}

/**
 * Summarise a command sequence by grouping waits and counting register writes.
 */
function summarise(commands) {
  let totalWait = 0;
  let writes = 0;
  for (const c of commands) {
    const cmd = c[0];
    if (cmd === 0x61) totalWait += c[1];
    else if (cmd === 0x62) totalWait += 735;
    else if (cmd === 0x63) totalWait += 882;
    else if (cmd >= 0x70 && cmd <= 0x7F) totalWait += (cmd & 0x0F) + 1;
    else if (cmd === 0x66) { /* end */ }
    else writes++;
  }
  return { totalWait, writes };
}

function compareSequences(ref, cur) {
  const diffs = [];
  const len = Math.max(ref.length, cur.length);
  let mismatches = 0;
  for (let i = 0; i < len; i++) {
    if (i >= ref.length) {
      diffs.push(`  + extra cmd @${i}: [${cur[i].map(x => '0x' + x.toString(16)).join(', ')}]`);
      mismatches++;
      if (mismatches > 20) { diffs.push('  ... (truncated)'); break; }
    } else if (i >= cur.length) {
      diffs.push(`  - missing cmd @${i}: [${ref[i].map(x => '0x' + x.toString(16)).join(', ')}]`);
      mismatches++;
      if (mismatches > 20) { diffs.push('  ... (truncated)'); break; }
    } else {
      const r = ref[i];
      const c = cur[i];
      if (r.length !== c.length || r.some((v, j) => v !== c[j])) {
        diffs.push(`  cmd @${i}: ref=[${r.map(x => '0x' + x.toString(16)).join(', ')}] cur=[${c.map(x => '0x' + x.toString(16)).join(', ')}]`);
        mismatches++;
        if (mismatches > 20) { diffs.push('  ... (truncated)'); break; }
      }
    }
  }
  return diffs;
}

function walkVgms(dir) {
  if (!fs.existsSync(dir)) return [];
  return fs.readdirSync(dir)
    .filter(f => f.toLowerCase().endsWith('.vgm'))
    .map(f => f.slice(0, -4))
    .sort();
}

// ---- main ----
const [,, refDir, curDir] = process.argv;
if (!refDir || !curDir) usage();

const refBases = new Set(walkVgms(refDir));
const curBases = new Set(walkVgms(curDir));

if (refBases.size === 0) {
  console.error(`No reference VGMs found in: ${refDir}`);
  process.exit(1);
}

let passed = 0;
let failed = 0;
let errors = 0;

for (const base of [...refBases].sort()) {
  const refPath = path.join(refDir, base + '.vgm');
  const curPath = path.join(curDir, base + '.vgm');

  if (!curBases.has(base)) {
    console.log(`MISSING  ${base}: no current VGM`);
    failed++;
    continue;
  }

  let refParsed, curParsed;
  try {
    refParsed = parseVgm(fs.readFileSync(refPath), refPath);
    curParsed = parseVgm(fs.readFileSync(curPath), curPath);
  } catch (e) {
    console.log(`ERROR    ${base}: ${e.message}`);
    errors++;
    continue;
  }

  const refSummary = summarise(refParsed.commands);
  const curSummary = summarise(curParsed.commands);

  const diffs = compareSequences(refParsed.commands, curParsed.commands);

  if (diffs.length === 0) {
    console.log(`PASS     ${base}  (${refSummary.writes} writes, ${refSummary.totalWait} samples)`);
    passed++;
  } else {
    console.log(`FAIL     ${base}`);
    console.log(`         ref: ${refSummary.writes} writes, ${refSummary.totalWait} wait-samples`);
    console.log(`         cur: ${curSummary.writes} writes, ${curSummary.totalWait} wait-samples`);
    diffs.forEach(d => console.log(d));
    failed++;
  }
}

// Report VGMs in current but not in reference
for (const base of [...curBases].sort()) {
  if (!refBases.has(base)) {
    console.log(`EXTRA    ${base}: in current but not in reference`);
  }
}

console.log('');
console.log(`Results: ${passed} passed, ${failed} failed, ${errors} errors`);
if (failed > 0 || errors > 0) process.exit(1);
