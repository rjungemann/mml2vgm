import type * as monaco from 'monaco-editor';

export function registerMmlLanguage(monacoInstance: typeof monaco): void {
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
      'OPNA', 'OPNB', 'SSG', 'ADPCM', 'PCM', 'rhythm',
      'o', 'l', 'v', 't', 'n', 'q',
      'r', 'w', 's', 'i', 'm', 'k', 'd', 'y',
      'T', 'V', 'Q', 'Y', 'X', 'W', 'P', 'L', 'M',
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

  // Define completion items
  monacoInstance.languages.registerCompletionItemProvider('mml', {
    provideCompletionItems: (model, position) => {
      const monaco = monacoInstance;
      const word = model.getWordAtPosition(position);
      const wordRange: monaco.IRange = word 
        ? { startLineNumber: position.lineNumber, startColumn: word.startColumn, endLineNumber: position.lineNumber, endColumn: word.endColumn }
        : { startLineNumber: position.lineNumber, startColumn: position.column, endLineNumber: position.lineNumber, endColumn: position.column };
      const suggestions: monaco.languages.CompletionItem[] = [
        { label: '@OPNA', kind: monaco.languages.CompletionItemKind.Keyword, insertText: '@OPNA', range: wordRange },
        { label: '@OPNB', kind: monaco.languages.CompletionItemKind.Keyword, insertText: '@OPNB', range: wordRange },
        { label: '@SSG', kind: monaco.languages.CompletionItemKind.Keyword, insertText: '@SSG', range: wordRange },
        { label: '@ADPCM', kind: monaco.languages.CompletionItemKind.Keyword, insertText: '@ADPCM', range: wordRange },
        { label: '@PCM', kind: monaco.languages.CompletionItemKind.Keyword, insertText: '@PCM', range: wordRange },
        { label: '@rhythm', kind: monaco.languages.CompletionItemKind.Keyword, insertText: '@rhythm', range: wordRange },
        { label: 'o4', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'o4', range: wordRange },
        { label: 'l4', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'l4', range: wordRange },
        { label: 'v100', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'v100', range: wordRange },
        { label: 't120', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 't120', range: wordRange },
        { label: 'C', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'C', range: wordRange },
        { label: 'D', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'D', range: wordRange },
        { label: 'E', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'E', range: wordRange },
        { label: 'F', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'F', range: wordRange },
        { label: 'G', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'G', range: wordRange },
        { label: 'A', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'A', range: wordRange },
        { label: 'B', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'B', range: wordRange },
        { label: 'r', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'r', range: wordRange },
      ];
      
      return { suggestions };
    },
  });
}
