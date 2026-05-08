#!/usr/bin/env node
/**
 * convert_sitaraba.mjs
 *
 * Converts Sitaraba / 3MLE MML format (.txt) to mml2vgm .gwi format.
 *
 * Key differences handled:
 *   - Sitaraba: note letter followed by number = DURATION  (e.g. c4 = C quarter)
 *   - mml2vgm:  note letter followed by number = OCTAVE    (e.g. c4 = C at octave 4)
 *     → explicit note durations are converted to l<N> commands
 *   - Sitaraba: '&' before second note = tie from previous
 *     → converted to '_' appended to the first note
 *   - Sitaraba: l<N>. sets a dotted default length
 *     → mml2vgm does not support dotted l; the dot is applied per-note when using the default
 *   - Sitaraba: n<N> MIDI note command (same semantics as mml2vgm, no conversion needed)
 *   - Sitaraba: comma-separated tracks → separate 'A1 'A2 'A3 part lines
 *   - Sitaraba: [Title]/[Source]/[Editor] header → '{ ... } song info block
 *
 * Usage:
 *   node scripts/convert_sitaraba.mjs examples/cyris/rumia.txt [...]
 */

import { readFileSync, writeFileSync } from 'fs';
import { basename } from 'path';

// ---------------------------------------------------------------------------
// Parsing the Sitaraba file format
// ---------------------------------------------------------------------------

function parseSitaraba(text) {
  const lines = text.replace(/\r\n/g, '\n').replace(/\r/g, '\n').split('\n');
  const meta = {};
  const mmlLines = [];
  let inMml = false;

  for (const line of lines) {
    if (!inMml) {
      const m = line.match(/^\[Title\]\s*(.+)/);   if (m) { meta.title  = m[1].trim(); continue; }
      const s = line.match(/^\[Source\]\s*(.+)/);  if (s) { meta.source = s[1].trim(); continue; }
      const e = line.match(/^\[Editor\]\s*(.+)/);  if (e) { meta.editor = e[1].trim(); continue; }
      if (line.trim() === 'MML@') { inMml = true; continue; }
    } else {
      const trimmed = line.trim();
      if (trimmed === ';' || trimmed === '; ') break;
      mmlLines.push(line);
    }
  }

  const mmlContent = mmlLines.join('');
  const tracks = splitTracks(mmlContent).map(t => t.trim()).filter(t => t.length > 0);
  return { meta, tracks };
}

/** Split comma-separated tracks, respecting loop brackets/parens. */
function splitTracks(content) {
  const tracks = [];
  let depth = 0;
  let current = '';
  for (const ch of content) {
    if      (ch === '[' || ch === '(') { depth++; current += ch; }
    else if (ch === ']' || ch === ')') { depth--; current += ch; }
    else if (ch === ',' && depth === 0) { tracks.push(current); current = ''; }
    else { current += ch; }
  }
  if (current.trim()) tracks.push(current);
  return tracks;
}

// ---------------------------------------------------------------------------
// Tokenizer – converts a Sitaraba MML string to a token list
// ---------------------------------------------------------------------------

function tokenizeMml(mml) {
  const tokens = [];
  let i = 0;
  const len = mml.length;

  function readDigits() {
    let s = '';
    while (i < len && mml[i] >= '0' && mml[i] <= '9') s += mml[i++];
    return s;
  }

  while (i < len) {
    const ch = mml[i];

    // Skip whitespace
    if (ch === ' ' || ch === '\t' || ch === '\n' || ch === '\r') { i++; continue; }

    const lc = ch.toLowerCase();

    // Tempo: t<N>
    if (lc === 't') {
      i++;
      const num = readDigits();
      if (num) tokens.push({ type: 'tempo', value: parseInt(num) });
      continue;
    }

    // Default length: l<N>[.]
    if (lc === 'l') {
      i++;
      const num = readDigits();
      let dotted = false;
      if (i < len && mml[i] === '.') { dotted = true; i++; }
      if (num) tokens.push({ type: 'length', value: parseInt(num), dotted });
      continue;
    }

    // Octave absolute: o<N>
    if (lc === 'o') {
      i++;
      const num = readDigits();
      if (num) tokens.push({ type: 'octave', value: parseInt(num) });
      continue;
    }

    // Octave shifts
    if (ch === '>') { tokens.push({ type: 'octaveUp' });   i++; continue; }
    if (ch === '<') { tokens.push({ type: 'octaveDown' }); i++; continue; }

    // Volume: v<N>
    if (lc === 'v') {
      i++;
      const num = readDigits();
      if (num) tokens.push({ type: 'volume', value: parseInt(num) });
      continue;
    }

    // Tie marker ('&' comes BEFORE the second note in Sitaraba)
    if (ch === '&') { tokens.push({ type: 'tie' }); i++; continue; }

    // MIDI note number: n<N>[.]  — must be checked before note letters (n is not a-g)
    if (lc === 'n') {
      i++;
      const num = readDigits();
      if (num) {
        let dotted = false;
        if (i < len && mml[i] === '.') { dotted = true; i++; }
        tokens.push({ type: 'midiNote', midi: parseInt(num), dotted, tied: false });
      }
      // If no digits follow, 'n' was not a valid command – skip
      continue;
    }

    // Rest: r[<N>][.]
    if (lc === 'r') {
      i++;
      const num = readDigits();
      let dotted = false;
      if (i < len && mml[i] === '.') { dotted = true; i++; }
      tokens.push({ type: 'rest', duration: num ? parseInt(num) : null, dotted, tied: false });
      continue;
    }

    // Note: [a-g][+|-|#][<N>][.]
    if (lc >= 'a' && lc <= 'g') {
      const letter = lc;
      i++;
      let accidental = '';
      if      (i < len && (mml[i] === '+' || mml[i] === '#')) { accidental = '+'; i++; }
      else if (i < len && mml[i] === '-')                     { accidental = '-'; i++; }
      const num = readDigits();
      let dotted = false;
      if (i < len && mml[i] === '.') { dotted = true; i++; }
      tokens.push({ type: 'note', letter, accidental, duration: num ? parseInt(num) : null, dotted, tied: false });
      continue;
    }

    // Loop markers
    if (ch === '[') { tokens.push({ type: 'loopStart' });  i++; continue; }
    if (ch === ']') { tokens.push({ type: 'loopEnd' });    i++; continue; }
    if (ch === '(') { tokens.push({ type: 'repeatStart' }); i++; continue; }
    if (ch === ')') {
      i++;
      const num = readDigits();
      tokens.push({ type: 'repeatEnd', count: num ? parseInt(num) : 1 });
      continue;
    }

    // Bar line
    if (ch === '|') { tokens.push({ type: 'bar' }); i++; continue; }

    // Unknown character – skip
    i++;
  }

  return tokens;
}

// ---------------------------------------------------------------------------
// Tie conversion
//   Sitaraba: noteA & noteB  (& comes BEFORE the second note)
//   mml2vgm:  noteA_ noteB  (_ appended to FIRST note)
// ---------------------------------------------------------------------------

function processTies(tokens) {
  const result = [];
  for (const t of tokens) {
    if (t.type === 'tie') {
      // Find the most-recent note/rest/midiNote in result and mark it tied
      for (let j = result.length - 1; j >= 0; j--) {
        const tt = result[j];
        if (tt.type === 'note' || tt.type === 'rest' || tt.type === 'midiNote') {
          result[j] = { ...tt, tied: true };
          break;
        }
      }
      continue; // don't push the '&' token itself
    }
    result.push(t);
  }
  return result;
}

// ---------------------------------------------------------------------------
// GWI emitter
//   In mml2vgm a number after a note letter is the OCTAVE, not the duration.
//   We must use l<N> commands to express durations.
//
//   Algorithm:
//     - Track sita_default_len / sita_default_dotted: what Sitaraba's current l<N>[.] is
//     - Track gwi_current_len:                        what we have actually emitted as l<N>
//     - For each note/rest:
//         effective_len    = note.duration  ?? sita_default_len
//         effective_dotted = (note.duration != null) ? note.dotted : (note.dotted || sita_default_dotted)
//         If effective_len != gwi_current_len → emit l<effective_len>
//         Emit bare note letter + accidental + [.] + [_]
// ---------------------------------------------------------------------------

function emitGwiTrack(tokens, injectTempo) {
  const parts = [];
  let sita_default_len    = 4;    // Sitaraba's current default note length
  let sita_default_dotted = false;// whether l<N>. is in effect
  let gwi_current_len     = 4;    // what l<N> we have emitted so far

  function ensureLen(len) {
    if (len !== gwi_current_len) {
      parts.push(`l${len}`);
      gwi_current_len = len;
    }
  }

  if (injectTempo != null) parts.push(`t${injectTempo}`);

  for (const token of tokens) {
    switch (token.type) {
      case 'tempo':
        parts.push(`t${token.value}`);
        break;

      case 'length':
        sita_default_len    = token.value;
        sita_default_dotted = token.dotted;
        ensureLen(token.value);
        // Note: mml2vgm doesn't support dotted l commands; the dot is applied per-note below.
        break;

      case 'octave':
        parts.push(`o${token.value}`);
        break;

      case 'octaveUp':  parts.push('>'); break;
      case 'octaveDown': parts.push('<'); break;

      case 'volume':
        parts.push(`v${token.value}`);
        break;

      case 'note': {
        const { letter, accidental, duration, dotted, tied } = token;
        const effLen    = duration !== null ? duration : sita_default_len;
        const effDotted = duration !== null ? dotted   : (dotted || sita_default_dotted);
        ensureLen(effLen);
        let s = letter + (accidental || '');
        if (effDotted) s += '.';
        if (tied)      s += '_';
        parts.push(s);
        break;
      }

      case 'rest': {
        const { duration, dotted, tied } = token;
        const effLen    = duration !== null ? duration : sita_default_len;
        const effDotted = duration !== null ? dotted   : (dotted || sita_default_dotted);
        ensureLen(effLen);
        let s = 'r';
        if (effDotted) s += '.';
        if (tied)      s += '_';
        parts.push(s);
        break;
      }

      case 'midiNote': {
        // n<N> uses the current default length in both formats.
        // Ensure gwi's l is synced to sita_default_len before emitting.
        ensureLen(sita_default_len);
        const effDotted = token.dotted || sita_default_dotted;
        let s = `n${token.midi}`;
        if (effDotted) s += '.';
        if (token.tied) s += '_';
        parts.push(s);
        break;
      }

      case 'loopStart':  parts.push('['); break;
      case 'loopEnd':    parts.push(']'); break;
      case 'repeatStart': parts.push('('); break;
      case 'repeatEnd':
        parts.push(')');
        if (token.count !== 1) parts.push(`${token.count}`);
        break;
      case 'bar': parts.push('|'); break;
    }
  }

  return parts.join('');
}

// ---------------------------------------------------------------------------
// Top-level conversion
// ---------------------------------------------------------------------------

function convertFile(text) {
  const { meta, tracks } = parseSitaraba(text);

  // Extract global tempo from the first track (Sitaraba always puts it there)
  const firstTokens = tokenizeMml(tracks[0] || '');
  const tempoToken  = firstTokens.find(t => t.type === 'tempo');
  const globalTempo = tempoToken ? tempoToken.value : null;

  // Build song info block
  let gwi = "'{\n";
  gwi += `    TitleName   = ${meta.title  || 'Unknown'}\n`;
  gwi += `    Composer    = ${meta.source || 'Unknown'}\n`;
  if (meta.editor) gwi += `    Arranger    = ${meta.editor}\n`;
  gwi += '    SystemName  = Generic SSG\n';
  gwi += '    Format      = VGM\n';
  gwi += '    ClockCount  = 192\n';
  gwi += '\n';
  gwi += '    PartAY8910  = A\n';
  gwi += "}\n\n";
  gwi += '; Converted from Sitaraba / 3MLE MML format by scripts/convert_sitaraba.mjs\n';
  gwi += '; Note: l<N>. (dotted default length) is expanded to per-note dots.\n\n';

  const partNames = ['A1', 'A2', 'A3', 'A4'];

  for (let i = 0; i < Math.min(tracks.length, 4); i++) {
    const partName = partNames[i];
    let tokens = tokenizeMml(tracks[i]);
    tokens = processTies(tokens);

    // Inject global tempo into parts 2+ if they don't have their own
    const hasTempo    = tokens.some(t => t.type === 'tempo');
    const injectTempo = (!hasTempo && globalTempo != null && i > 0) ? globalTempo : null;

    const content = emitGwiTrack(tokens, injectTempo);
    gwi += `'${partName} ${content}\n`;
  }

  return gwi;
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

const args = process.argv.slice(2);
if (args.length === 0) {
  console.error('Usage: node scripts/convert_sitaraba.mjs <input.txt> [...]');
  process.exit(1);
}

for (const inputPath of args) {
  try {
    const text     = readFileSync(inputPath, 'utf8');
    const gwi      = convertFile(text);
    const outPath  = inputPath.replace(/\.txt$/, '.gwi');
    writeFileSync(outPath, gwi);
    console.log(`Converted: ${basename(inputPath)} → ${basename(outPath)}`);
  } catch (err) {
    console.error(`Error processing ${inputPath}: ${err.message}`);
    process.exit(1);
  }
}
