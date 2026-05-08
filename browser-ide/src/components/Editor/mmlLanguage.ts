import type * as monaco from 'monaco-editor';
import { driverService } from '@/services/driverService';
import type { CompletionSuggestion } from '@/services/driverService';

/**
 * Register the 'mml' Monaco language with syntax highlighting and
 * a dynamic, per-driver completion provider.
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
    extensions: ['.mml', '.txt'],
    aliases: ['MML', 'mml', 'Music Macro Language'],
    mimetypes: ['text/x-mml'],
  });

  // Define token types for MML syntax highlighting
  monacoInstance.languages.setMonarchTokensProvider('mml', {
    defaultToken: 'invalid',
    
    ignoreCase: true,
    
    // Keywords
    keywords: [
      '@',
      // Full-tier chips
      'OPNA', 'OPNB', 'SSG', 'ADPCM', 'PCM', 'rhythm',
      'o', 'l', 'v', 't', 'n', 'q',
      'r', 'w', 's', 'i', 'm', 'k', 'd', 'y',
      'T', 'V', 'Q', 'Y', 'X', 'W', 'P', 'L', 'M',
      // Partial chip Part* keywords (Batch 1: Sega & FM Core)
      'PartYM2608', 'PartYM2608FM', 'PartYM2608SSG', 'PartYM2608ADPCM',
      'PartYM2151', 'PartYM2151FM', 'PartOPM',
      'PartYM2203', 'PartYM2203FM', 'PartYM2203SSG', 'PartOPN',
      'PartRF5C164',
      'PartSegaPCM',
      // Partial chip Part* keywords (Batch 2: OPL Family)
      'PartYM3526', 'PartOPL',
      'PartY8950',
      'PartYM3812', 'PartOPL2',
      'PartYMF262', 'PartOPL3',
      // Partial chip Part* keywords (Batch 3: Console PSG/FM)
      'PartYM2413', 'PartOPLL',
      'PartHuC6280',
      'PartNES', 'PartNESPulse', 'PartNESTriangle', 'PartNESNoise', 'PartNESDPCM',
      'PartDMG', 'PartDMGPulse', 'PartDMGWave', 'PartDMGNoise',
      // Partial chip Part* keywords (Batch 4: Arcade PCM)
      'PartC140',
      'PartC352',
      'PartK053260',
      'PartK054539',
      // Partial chip Part* keywords (Batch 5: Miscellaneous)
      'PartAY8910',
      'PartK051649', 'PartSCC', 'PartSCC1',
      'PartPOKEY',
      'PartVRC6',
      'PartQSound',
      // Phase 9: Chip-Specific Commands
      // FM Operator Commands
      'AR', 'DR', 'SR', 'RR', 'SL', 'TL', 'KS', 'ML', 'DT',
      'AL', 'FB', 'OP',
      // OPL3 / Special FM Commands
      'OPL3MODE', '4OP', 'CUSTOM', 'VIB', 'TREM', 'DRUM',
      // PSG / AY8910 / POKEY Commands
      'EN', 'MIX', 'FILTER', 'DIST', 'HPOLY', 'NOISE',
      // Wavetable Commands
      'WAVE', 'NW', 'SW', 'KEYON', 'KEYOFF', 'NOCTRL',
      // PCM Commands
      'BANK', 'START', 'LOOP', 'END', 'REVERSE', 'LOOPSTART', 'LOOPLEN',
      'LVOL', 'RVOL', 'ADPCM_MODE', 'PAN', 'REVERB', 'PITCH', 'VOLUME',
      // Metadata
      'ForcedMonoPartYM2612',
    ],
    
    // Built-in functions/operations
    operators: [
      '=', '>', '<', '!', '&', '|', '+', '-', '*', '/', '%',
    ],
    
    // Token patterns
    tokenizer: {
      root: [
        // Comments
        { regex: /;.*$/, action: { token: 'comment.line' } },
        { regex: /\/\/.*$/, action: { token: 'comment.line' } },
        { regex: /\/\*/, action: { token: 'comment.block', next: 'comment' } },
        
        // Strings
        { regex: /"([^"\\]|\\.)*"/, action: { token: 'string' } },
        { regex: /'([^'\\]|\\.)*'/, action: { token: 'string' } },
        
        // Channel specifiers
        { regex: /@[0-9]+/i, action: { token: 'keyword.channel' } },
        
        // Note names
        { regex: /[A-G][#b]?[0-9]*/, action: { token: 'keyword.note' } },
        
        // Rest
        { regex: /r[0-9]*/i, action: { token: 'keyword.rest' } },
        
        // Octave
        { regex: /o[0-9]+/i, action: { token: 'keyword.octave' } },
        
        // Length
        { regex: /l[0-9]+/i, action: { token: 'keyword.length' } },
        
        // Volume
        { regex: /v[0-9]+/i, action: { token: 'keyword.volume' } },
        
        // Tempo
        { regex: /t[0-9]+/i, action: { token: 'keyword.tempo' } },
        
        // Quantum
        { regex: /q[0-9]+/i, action: { token: 'keyword.quantum' } },
        
        // Tie
        { regex: /&/, action: { token: 'keyword.tie' } },
        
        // Slur
        { regex: /~/, action: { token: 'keyword.slur' } },
        
        // Dots
        { regex: /\./, action: { token: 'keyword.dot' } },
        
        // Numbers
        { regex: /\$[0-9a-fA-F]+/i, action: { token: 'number.hex' } },
        { regex: /0[xX][0-9a-fA-F]+/i, action: { token: 'number.hex' } },
        { regex: /[0-9]+(\.[0-9]+)?/, action: { token: 'number' } },
        
        // Punctuation
        { regex: /[,;:{}\(\)\[\]]/, action: { token: 'delimiter' } },
        
        // Whitespace
        { regex: /\s+/, action: { token: 'white' } },
        
        // Identifiers
        { regex: /[a-zA-Z_][a-zA-Z0-9_]*/, action: {
          cases: {
            '@keywords': { token: 'keyword' },
            '@default': { token: 'identifier' },
          },
        }},
      ],
      
      comment: [
        { regex: /\*\//, action: { token: 'comment.block', next: '@pop' } },
        { regex: /.*/, action: { token: 'comment.block' } },
      ],
    },
  });

  // Define language configuration
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
      { open: "'", close: "'" },
    ],
    surroundingPairs: [
      { open: '(', close: ')' },
      { open: '[', close: ']' },
      { open: '{', close: '}' },
      { open: '"', close: '"' },
      { open: "'", close: "'" },
    ],
    wordPattern: /[a-zA-Z0-9_#$]/,
    indentationRules: {
      increaseIndentPattern: /^\s*[A-Za-z_@]/,
      decreaseIndentPattern: /^\s*[}\]]/,
    },
  });

  // Define completion items — delegate to per-driver completions from driverService
  monacoInstance.languages.registerCompletionItemProvider('mml', {
    triggerCharacters: ['@', '#', "'"],
    provideCompletionItems: (model, position) => {
      const monacoNs = monacoInstance;
      const word = model.getWordAtPosition(position);
      const wordRange: monaco.IRange = word
        ? { startLineNumber: position.lineNumber, startColumn: word.startColumn, endLineNumber: position.lineNumber, endColumn: word.endColumn }
        : { startLineNumber: position.lineNumber, startColumn: position.column, endLineNumber: position.lineNumber, endColumn: position.column };

      // Determine prefix for filtering (include trigger chars @ and # if present)
      const lineUpToCursor = model.getValueInRange({
        startLineNumber: position.lineNumber,
        startColumn: 1,
        endLineNumber: position.lineNumber,
        endColumn: position.column,
      });
      const prefixMatch = lineUpToCursor.match(/[@#']?\w*$/);
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
