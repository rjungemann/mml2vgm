import React, { useEffect, useRef, useCallback, useState } from 'react';
import { useMonaco } from '@monaco-editor/react';
import Editor, { OnMount } from '@monaco-editor/react';
import type { Document, EditorSettings, Position } from '@/types';
import { registerMmlLanguage } from './mmlLanguage';
import { registerMmlTheme } from './mmlTheme';

interface MonacoEditorProps {
  document: Document;
  onChange: (content: string) => void;
  settings: EditorSettings;
  // Trace playback props
  currentPosition?: Position | null;
  // Navigation
  navigationPosition?: Position | null;
}

const MonacoEditor: React.FC<MonacoEditorProps> = ({
  document,
  onChange,
  settings,
  currentPosition,
  navigationPosition,
}) => {
  const editorRef = useRef<any>(null);
  const monaco = useMonaco();
  const languageId = 'mml';
  const [currentLineDecorations, setCurrentLineDecorations] = useState<string[]>([]);
  const [navDecorations, setNavDecorations] = useState<string[]>([]);

  // Register MML language and theme when Monaco is loaded
  useEffect(() => {
    if (!monaco) return;

    // Register MML language
    registerMmlLanguage(monaco);

    // Register themes
    registerMmlTheme(monaco, 'mml-dark', 'vs-dark');
    registerMmlTheme(monaco, 'mml-light', 'vs');

    // Set the current theme
    monaco.editor.setTheme(settings.theme === 'vs-dark' ? 'mml-dark' : 'mml-light');
  }, [monaco, settings.theme]);

  // Highlight current playback position
  useEffect(() => {
    if (!editorRef.current || !currentPosition) return;

    const editor = editorRef.current;
    const monacoInstance = editor._monaco;

    // Clear previous decorations
    if (currentLineDecorations.length > 0) {
      editor.deltaDecorations(currentLineDecorations, []);
    }

    // Highlight current line
    const lineNumber = currentPosition.line;
    const decorations = editor.deltaDecorations([], [
      {
        range: new monacoInstance.Range(lineNumber, 1, lineNumber, 1),
        options: {
          isWholeLine: true,
          className: 'trace-current-line',
          marginClassName: 'trace-current-line-gutter',
          glyphMarginClassName: 'trace-current-line-gutter',
        },
      },
    ]);

    setCurrentLineDecorations(decorations);

    // Scroll to current line (if auto-scroll is enabled)
    const lineHeight = editor.getOption(monacoInstance.editor.EditorOption.lineHeight);
    const scrollTop = (lineNumber - 1) * lineHeight - editor.getScrollTop();
    const clientHeight = editor.getLayoutInfo().clientHeight;
    
    // Only scroll if line is not visible
    if (scrollTop > clientHeight || scrollTop < 0) {
      editor.revealLineInCenter(lineNumber);
    }
  }, [currentPosition, currentLineDecorations]);

  // Handle navigation to a specific position (from error list click)
  useEffect(() => {
    if (!editorRef.current || !navigationPosition) return;

    const editor = editorRef.current;
    const monacoInstance = editor._monaco;

    // Clear previous navigation decorations
    if (navDecorations.length > 0) {
      editor.deltaDecorations(navDecorations, []);
    }

    // Set cursor position and reveal line
    const { line, column } = navigationPosition;
    editor.setPosition({ lineNumber: line, column: column });
    editor.revealLineInCenter(line);
    editor.focus();

    // Add temporary highlight for navigation
    const decorations = editor.deltaDecorations([], [
      {
        range: new monacoInstance.Range(line, column, line, column + 1),
        options: {
          className: 'navigate-highlight',
          isWholeLine: false,
        },
      },
    ]);

    setNavDecorations(decorations);

    // Clear navigation after a short delay
    const timer = setTimeout(() => {
      if (navDecorations.length > 0) {
        editor.deltaDecorations(navDecorations, []);
        setNavDecorations([]);
      }
    }, 1000);

    return () => clearTimeout(timer);
  }, [navigationPosition, navDecorations]);

  // Handle editor mount
  const handleEditorDidMount: OnMount = useCallback((editor, _monaco) => {
    editorRef.current = editor;

    // Set initial content
    editor.setValue(document.content);

    // Set editor options from settings
    editor.updateOptions({
      fontSize: settings.fontSize,
      fontFamily: settings.fontFamily,
      wordWrap: settings.wordWrap ? 'on' : 'off',
      minimap: { enabled: settings.showMinimap },
      scrollBeyondLastLine: false,
      renderLineHighlight: 'all',
      lineNumbers: 'on',
      glyphMargin: true,
      folding: true,
      lineDecorationsWidth: 10,
      lineNumbersMinChars: 3,
      selectOnLineNumbers: true,
      roundedSelection: true,
      cursorStyle: 'line',
      automaticLayout: true,
      ...settings.options,
    });

    // Listen for content changes
    const disposable = editor.onDidChangeModelContent(() => {
      const content = editor.getValue();
      onChange(content);
    });

    // Store disposable for cleanup
    return () => {
      disposable.dispose();
    };
  }, [document.content, onChange, settings]);

  // Update theme when settings change
  useEffect(() => {
    if (!monaco) return;

    const themeName = settings.theme === 'vs-dark' ? 'mml-dark' : 'mml-light';
    monaco.editor.setTheme(themeName);
  }, [monaco, settings.theme]);

  // Update editor options when settings change
  useEffect(() => {
    if (!editorRef.current) return;

    editorRef.current.updateOptions({
      fontSize: settings.fontSize,
      fontFamily: settings.fontFamily,
      wordWrap: settings.wordWrap ? 'on' : 'off',
      minimap: { enabled: settings.showMinimap },
    });
  }, [settings.fontSize, settings.fontFamily, settings.wordWrap, settings.showMinimap]);

  // Handle document content changes
  useEffect(() => {
    if (!editorRef.current) return;

    const currentValue = editorRef.current.getValue();
    if (currentValue !== document.content) {
      editorRef.current.setValue(document.content);
    }
  }, [document.content, document.id]);

  return (
    <div className="editor-wrapper">
      <Editor
        language={languageId}
        theme={settings.theme === 'vs-dark' ? 'mml-dark' : 'mml-light'}
        onMount={handleEditorDidMount}
        options={{
          fontSize: settings.fontSize,
          fontFamily: settings.fontFamily,
          wordWrap: settings.wordWrap ? 'on' : 'off',
          minimap: { enabled: settings.showMinimap },
          scrollBeyondLastLine: false,
          renderLineHighlight: 'all',
          lineNumbers: 'on',
          glyphMargin: true,
          folding: true,
          lineDecorationsWidth: 10,
          lineNumbersMinChars: 3,
          selectOnLineNumbers: true,
          roundedSelection: true,
          cursorStyle: 'line',
          automaticLayout: true,
          ...settings.options,
        }}
      />
    </div>
  );
};

export default MonacoEditor;
