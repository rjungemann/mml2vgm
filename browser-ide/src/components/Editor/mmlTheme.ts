import type * as monaco from 'monaco-editor';

export function registerMmlTheme(monacoInstance: typeof monaco, name: string, base: 'vs-dark' | 'vs' | 'hc-black'): void {
  const themeData: monaco.editor.IStandaloneThemeData = {
    base,
    inherit: true,
    rules: getThemeRules(base),
    colors: getThemeColors(base),
  };

  monacoInstance.editor.defineTheme(name, themeData);
}

function getThemeRules(base: string): monaco.editor.ITokenThemeRule[] {
  // Dark theme rules
  const darkRules: monaco.editor.ITokenThemeRule[] = [
    { token: 'comment.line', foreground: '6a9955', fontStyle: 'italic' },
    { token: 'comment.block', foreground: '6a9955', fontStyle: 'italic' },
    { token: 'string', foreground: 'ce9178' },
    { token: 'string.escape', foreground: 'd7ba7d' },
    { token: 'keyword', foreground: '569cd6', fontStyle: 'bold' },

    // Line-leading apostrophe constructs.
    { token: 'keyword.header', foreground: 'c586c0', fontStyle: 'bold' },
    { token: 'keyword.instrument', foreground: 'c586c0', fontStyle: 'bold' },
    { token: 'keyword.instType', foreground: '569cd6', fontStyle: 'bold' },
    { token: 'keyword.part', foreground: 'dcdcaa', fontStyle: 'bold' },

    // Header-block tokens.
    { token: 'attribute.name', foreground: '9cdcfe' },
    { token: 'type.chip', foreground: '4ec9b0', fontStyle: 'bold' },
    { token: 'type.fmParam', foreground: '4ec9b0' },
    { token: 'keyword.literal', foreground: '569cd6' },

    // Part-track tokens.
    { token: 'keyword.note', foreground: '4ec9b0', fontStyle: 'bold' },
    { token: 'keyword.rest', foreground: 'dcdcaa' },
    { token: 'keyword.octave', foreground: 'b5cea8' },
    { token: 'keyword.octave.shift', foreground: 'b5cea8', fontStyle: 'bold' },
    { token: 'keyword.length', foreground: 'b5cea8' },
    { token: 'keyword.length.global', foreground: 'b5cea8', fontStyle: 'bold' },
    { token: 'keyword.volume', foreground: 'd69d85' },
    { token: 'keyword.volume.global', foreground: 'd69d85', fontStyle: 'bold' },
    { token: 'keyword.tempo', foreground: 'd69d85' },
    { token: 'keyword.tempo.global', foreground: 'd69d85', fontStyle: 'bold' },
    { token: 'keyword.quantum', foreground: 'd69d85' },
    { token: 'keyword.quantum.global', foreground: 'd69d85', fontStyle: 'bold' },
    { token: 'keyword.tie', foreground: 'ce9178' },
    { token: 'keyword.slur', foreground: 'ce9178' },
    { token: 'keyword.dot', foreground: 'd4d4d4' },

    { token: 'number', foreground: 'b5cea8' },
    { token: 'number.hex', foreground: 'd7ba7d' },
    { token: 'identifier', foreground: '9cdcfe' },
    { token: 'delimiter', foreground: 'd4d4d4' },
    { token: 'delimiter.loop', foreground: 'c586c0', fontStyle: 'bold' },
    { token: 'delimiter.bar', foreground: '808080' },
    { token: 'operator', foreground: 'd4d4d4' },
    { token: 'white', foreground: 'd4d4d4' },
  ];

  // Light theme rules
  const lightRules: monaco.editor.ITokenThemeRule[] = [
    { token: 'comment.line', foreground: '6a737d', fontStyle: 'italic' },
    { token: 'comment.block', foreground: '6a737d', fontStyle: 'italic' },
    { token: 'string', foreground: '032f62' },
    { token: 'string.escape', foreground: '005cc5' },
    { token: 'keyword', foreground: 'd73a49', fontStyle: 'bold' },

    { token: 'keyword.header', foreground: 'a626a4', fontStyle: 'bold' },
    { token: 'keyword.instrument', foreground: 'a626a4', fontStyle: 'bold' },
    { token: 'keyword.instType', foreground: 'd73a49', fontStyle: 'bold' },
    { token: 'keyword.part', foreground: 'b58900', fontStyle: 'bold' },

    { token: 'attribute.name', foreground: '005cc5' },
    { token: 'type.chip', foreground: '22863a', fontStyle: 'bold' },
    { token: 'type.fmParam', foreground: '22863a' },
    { token: 'keyword.literal', foreground: 'd73a49' },

    { token: 'keyword.note', foreground: '22863a', fontStyle: 'bold' },
    { token: 'keyword.rest', foreground: '735c0f' },
    { token: 'keyword.octave', foreground: '005cc5' },
    { token: 'keyword.octave.shift', foreground: '005cc5', fontStyle: 'bold' },
    { token: 'keyword.length', foreground: '005cc5' },
    { token: 'keyword.length.global', foreground: '005cc5', fontStyle: 'bold' },
    { token: 'keyword.volume', foreground: '6f42c1' },
    { token: 'keyword.volume.global', foreground: '6f42c1', fontStyle: 'bold' },
    { token: 'keyword.tempo', foreground: '6f42c1' },
    { token: 'keyword.tempo.global', foreground: '6f42c1', fontStyle: 'bold' },
    { token: 'keyword.quantum', foreground: '6f42c1' },
    { token: 'keyword.quantum.global', foreground: '6f42c1', fontStyle: 'bold' },
    { token: 'keyword.tie', foreground: '032f62' },
    { token: 'keyword.slur', foreground: '032f62' },
    { token: 'keyword.dot', foreground: '24292e' },

    { token: 'number', foreground: '005cc5' },
    { token: 'number.hex', foreground: '005cc5' },
    { token: 'identifier', foreground: '005cc5' },
    { token: 'delimiter', foreground: '24292e' },
    { token: 'delimiter.loop', foreground: 'a626a4', fontStyle: 'bold' },
    { token: 'delimiter.bar', foreground: '959da5' },
    { token: 'operator', foreground: 'd73a49' },
    { token: 'white', foreground: '24292e' },
  ];

  return base === 'vs-dark' ? darkRules : lightRules;
}

function getThemeColors(base: string): Record<string, string> {
  // Dark theme colors
  const darkColors: Record<string, string> = {
    'editor.background': '#1e1e1e',
    'editor.foreground': '#d4d4d4',
    'editorCursor.foreground': '#ffffff',
    'editor.lineHighlightBackground': '#264f78',
    'editorLineNumber.foreground': '#6e7681',
    'editor.selectionBackground': '#264f78',
    'editor.inactiveSelectionBackground': '#3f3f46',
    'editorBracketMatch.background': '#264f78',
    'editorBracketMatch.border': '#569cd6',
    'editorIndentGuide.background': '#404040',
    'editorIndentGuide.activeBackground': '#707070',
    'statusBar.background': '#007acc',
    'statusBar.foreground': '#ffffff',
  };

  // Light theme colors
  const lightColors: Record<string, string> = {
    'editor.background': '#ffffff',
    'editor.foreground': '#24292e',
    'editorCursor.foreground': '#000000',
    'editor.lineHighlightBackground': '#c8e1ff',
    'editorLineNumber.foreground': '#959da5',
    'editor.selectionBackground': '#0366d630',
    'editor.inactiveSelectionBackground': '#0366d615',
    'editorBracketMatch.background': '#c8e1ff',
    'editorBracketMatch.border': '#0366d6',
    'editorIndentGuide.background': '#d3d3d3',
    'editorIndentGuide.activeBackground': '#959da5',
    'statusBar.background': '#0366d6',
    'statusBar.foreground': '#ffffff',
  };

  return base === 'vs-dark' ? darkColors : lightColors;
}
