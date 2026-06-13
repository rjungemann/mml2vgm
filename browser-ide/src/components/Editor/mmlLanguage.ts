import type * as monaco from 'monaco-editor';
import { driverService } from '@/services/driverService';
import type { CompletionSuggestion } from '@/services/driverService';

/**
 * Register the 'mml' Monaco language with syntax highlighting and
 * a dynamic, per-driver completion provider.
 *
 * The grammar reflects the C# mml2vgm dialect actually accepted by the
 * compiler, not the hallucinated `@OPNA`/`@0` dialect of older sample files.
 * Real MML lines look like:
 *
 *   '{                                  ← header block open
 *       TitleName   = FM Basics
 *       PartYM2612  = A
 *   }                                   ← header block close
 *
 *   '@ M 000                            ← FM instrument definition label
 *      AR  DR  SR  RR  SL  TL  KS …    ← optional column-header row
 *   '@ 031,012,000,007,002,000, …       ← operator parameter row
 *   '@ 007,000                          ← ALG, FB row
 *
 *   'A1 T120 @0 v100 l4 o4 c d e f >c   ← part track line
 *
 * Highlighting is state-based:
 *   - `root`            handles line-leading apostrophe constructs and comments
 *   - `header`          entered on `'{`, exited on `}` (may span many lines)
 *   - `instrumentLine`  entered on `'@`, lasts until end of line
 *   - `partLine`        entered on `'A1` etc., lasts until end of line
 *
 * @param monacoInstance  The monaco namespace (from @monaco-editor/react)
 * @param getDriverId     A callback returning the active document's language/driver ID
 *                        (e.g. 'gwi', 'muc', 'mus', 'mdl', …).  Called at completion time
 *                        so it always reflects the current document.
 */
export function registerMmlLanguage(
  monacoInstance: typeof monaco,
  getDriverId: () => string,
): void {
  monacoInstance.languages.register({
    id: 'mml',
    extensions: ['.mml', '.gwi', '.muc', '.mus', '.mdl', '.txt'],
    aliases: ['MML', 'mml', 'Music Macro Language'],
    mimetypes: ['text/x-mml'],
  });

  monacoInstance.languages.setMonarchTokensProvider('mml', {
    defaultToken: '',

    // FM operator parameter names — appear in the unindented column-header row
    // immediately after `'@ M nnn`. Highlighted everywhere for simplicity.
    fmParams: [
      'AR', 'DR', 'SR', 'RR', 'SL', 'TL', 'KS', 'ML', 'DT',
      'AM', 'SSG-EG', 'AL', 'FB', 'OP',
    ],

    // Header-block identifiers (left-hand side of `=` lines inside `'{ … }`).
    headerKeys: [
      'TitleName', 'SystemName', 'Composer', 'Format',
      'ClockCount', 'Octave-Rev', 'VolumeUpDown', 'Version',
      'LoopMode', 'JumpToLoop',
    ],

    // Chip-binding keys (also LHS in the header). Listed separately so they
    // can take a distinct colour from informational keys.
    chipKeys: [
      'PartYM2612', 'PartYM2608', 'PartYM2608FM', 'PartYM2608SSG', 'PartYM2608ADPCM',
      'PartYM2151', 'PartYM2151FM', 'PartOPM',
      'PartYM2203', 'PartYM2203FM', 'PartYM2203SSG', 'PartOPN',
      'PartYM2413', 'PartOPLL',
      'PartYM3526', 'PartOPL',
      'PartY8950',
      'PartYM3812', 'PartOPL2',
      'PartYMF262', 'PartOPL3',
      'PartYMF271', 'PartOPL4',
      'PartSN76489', 'PartDCSG',
      'PartHuC6280',
      'PartNES', 'PartNESPulse', 'PartNESTriangle', 'PartNESNoise', 'PartNESDPCM',
      'PartDMG', 'PartDMGPulse', 'PartDMGWave', 'PartDMGNoise',
      'PartRF5C164', 'PartSegaPCM',
      'PartC140', 'PartC352',
      'PartK053260', 'PartK054539',
      'PartAY8910',
      'PartK051649', 'PartSCC', 'PartSCC1',
      'PartPOKEY', 'PartVRC6', 'PartQSound',
      'ForcedMonoPartYM2612',
    ],

    tokenizer: {
      root: [
        // Line comments
        [/;.*$/, 'comment.line'],
        [/\/\/.*$/, 'comment.line'],
        [/\/\*/, { token: 'comment.block', next: '@blockComment' }],

        // Header block: opens with `'{`, closes with `}` (no leading apostrophe).
        [/'\{/, { token: 'keyword.header', next: '@header' }],

        // FM / PCM / waveform instrument definition line — runs to end of line.
        [/'@/, { token: 'keyword.instrument', next: '@instrumentLine' }],

        // Part-track line: `'A1`, `'B2`, ... Apostrophe + letter + digits.
        [/'[A-Za-z][0-9]+/, { token: 'keyword.part', next: '@partLine' }],

        // Stand-alone apostrophe (shouldn't appear in valid MML; render plainly).
        [/'/, 'delimiter'],

        // Bare FM column-header row (e.g. `   AR  DR  SR  RR  SL  TL  KS …`).
        [/\b(?:AR|DR|SR|RR|SL|TL|KS|ML|DT|AM|SSG-EG|AL|FB|OP)\b/, 'type.fmParam'],

        [/\s+/, 'white'],
      ],

      // Inside `'{ … }` header — may span many lines until the closing `}`.
      header: [
        [/;.*$/, 'comment.line'],
        [/\}/, { token: 'keyword.header', next: '@pop' }],

        // `LHS = RHS` lines.
        [/[A-Za-z][A-Za-z0-9_-]*/, {
          cases: {
            '@chipKeys': 'type.chip',
            '@headerKeys': 'attribute.name',
            '@default': 'identifier',
          },
        }],
        [/=/, 'operator'],

        // Header values: booleans, identifiers, numbers, strings.
        [/\b(?:TRUE|FALSE|VGM|XGM|S98)\b/, 'keyword.literal'],
        [/"([^"\\]|\\.)*"/, 'string'],
        [/[0-9]+/, 'number'],

        [/\s+/, 'white'],
      ],

      // Single-line instrument-definition state (entered after `'@`).
      instrumentLine: [
        [/;.*$/, { token: 'comment.line', next: '@pop' }],
        [/$/, { token: '', next: '@pop' }],

        // Instrument-type label: M = FM melody, F = FM custom, P = PCM, etc.
        [/\b[MFPNRWQ]\b/, 'keyword.instType'],

        // FM operator parameter names occasionally appear inline.
        [/\b(?:AR|DR|SR|RR|SL|TL|KS|ML|DT|AM|SSG-EG|AL|FB|OP)\b/, 'type.fmParam'],

        [/[0-9]+/, 'number'],
        [/,/, 'delimiter'],
        [/\s+/, 'white'],
      ],

      // Single-line part-track state (entered after `'A1`, `'B2`, etc.).
      partLine: [
        [/;.*$/, { token: 'comment.line', next: '@pop' }],
        [/$/, { token: '', next: '@pop' }],

        // Global / clock-tick commands — uppercase.
        [/T[0-9]+/, 'keyword.tempo.global'],
        [/V[0-9]+/, 'keyword.volume.global'],
        [/L[0-9]+\.?/, 'keyword.length.global'],
        [/Q[0-9]+/, 'keyword.quantum.global'],

        // Instrument select.
        [/@[0-9]+/, 'keyword.instrument'],

        // Per-part commands — lowercase.
        [/o[0-9]+/, 'keyword.octave'],
        [/l[0-9]+\.?/, 'keyword.length'],
        [/v[0-9]+/, 'keyword.volume'],
        [/t[0-9]+/, 'keyword.tempo'],
        [/q[0-9]+/, 'keyword.quantum'],

        // Rests.
        [/r[0-9]*\.?/, 'keyword.rest'],

        // Note: a-g, optional accidental (+, -, #), optional length, optional dot.
        [/[a-g][+\-#]?[0-9]*\.?/, 'keyword.note'],

        // Octave shifts.
        [/[><]/, 'keyword.octave.shift'],

        // Tie / slur / dot.
        [/&/, 'keyword.tie'],
        [/~/, 'keyword.slur'],
        [/\./, 'keyword.dot'],

        // Loop brackets — `(... )N` where N is the repeat count.
        [/\(/, 'delimiter.loop'],
        [/\)[0-9]*/, 'delimiter.loop'],

        // Bar line — informational, ignored by compiler.
        [/\|/, 'delimiter.bar'],

        // Loop marker — declares the VGM loop point. Codegen populates
        // header offsets 0x1C/0x20 from this; player wraps back when end of
        // stream is reached with looping enabled.
        [/\$/, 'keyword.loopMarker'],

        // Hex / decimal numbers (for raw register writes etc.).
        [/\$[0-9a-fA-F]+/, 'number.hex'],
        [/0[xX][0-9a-fA-F]+/, 'number.hex'],
        [/[0-9]+/, 'number'],

        [/,/, 'delimiter'],
        [/[{}\[\]]/, 'delimiter'],
        [/\s+/, 'white'],
      ],

      blockComment: [
        [/\*\//, { token: 'comment.block', next: '@pop' }],
        [/[^*]+/, 'comment.block'],
        [/\*/, 'comment.block'],
      ],
    },
  } as monaco.languages.IMonarchLanguage);

  monacoInstance.languages.setLanguageConfiguration('mml', {
    comments: {
      lineComment: ';',
      blockComment: ['/*', '*/'],
    },
    brackets: [
      ['(', ')'],
      ['[', ']'],
      ['{', '}'],
    ],
    autoClosingPairs: [
      { open: '(', close: ')' },
      { open: '[', close: ']' },
      { open: '{', close: '}' },
      { open: '"', close: '"' },
    ],
    surroundingPairs: [
      { open: '(', close: ')' },
      { open: '[', close: ']' },
      { open: '{', close: '}' },
      { open: '"', close: '"' },
    ],
    // Identifier word characters include `-` so `Octave-Rev` is one word.
    wordPattern: /[A-Za-z_][A-Za-z0-9_-]*|[#$][0-9a-fA-F]+/,
    indentationRules: {
      increaseIndentPattern: /^\s*'?\{\s*$/,
      decreaseIndentPattern: /^\s*\}\s*$/,
    },
  });

  // Completion provider — delegate to per-driver completions from driverService.
  monacoInstance.languages.registerCompletionItemProvider('mml', {
    triggerCharacters: ['@', '#', "'"],
    provideCompletionItems: (model, position) => {
      const monacoNs = monacoInstance;
      const word = model.getWordAtPosition(position);
      const wordRange: monaco.IRange = word
        ? { startLineNumber: position.lineNumber, startColumn: word.startColumn, endLineNumber: position.lineNumber, endColumn: word.endColumn }
        : { startLineNumber: position.lineNumber, startColumn: position.column, endLineNumber: position.lineNumber, endColumn: position.column };

      const lineUpToCursor = model.getValueInRange({
        startLineNumber: position.lineNumber,
        startColumn: 1,
        endLineNumber: position.lineNumber,
        endColumn: position.column,
      });
      const prefixMatch = lineUpToCursor.match(/[@#']?[\w-]*$/);
      const prefix = prefixMatch ? prefixMatch[0] : (word?.word ?? '');

      const driverId = getDriverId();
      const rawSuggestions: CompletionSuggestion[] = driverService.getCompletions(driverId, prefix);

      const kindMap: Record<CompletionSuggestion['kind'], monaco.languages.CompletionItemKind> = {
        keyword: monacoNs.languages.CompletionItemKind.Keyword,
        snippet: monacoNs.languages.CompletionItemKind.Snippet,
        value: monacoNs.languages.CompletionItemKind.Value,
      };

      const suggestions: monaco.languages.CompletionItem[] = rawSuggestions.map(s => ({
        label: s.label,
        kind: kindMap[s.kind],
        insertText: s.insertText,
        detail: s.detail,
        documentation: s.documentation,
        insertTextRules: s.kind === 'snippet'
          ? monacoNs.languages.CompletionItemInsertTextRule.InsertAsSnippet
          : undefined,
        range: wordRange,
      }));

      return { suggestions };
    },
  });
}
