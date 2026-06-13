import { useEffect, useRef, useCallback, useImperativeHandle, forwardRef } from 'react';
import { useMonaco } from '@monaco-editor/react';
import Editor, { OnMount } from '@monaco-editor/react';
import type { Document, EditorSettings, Position, SourceMapEvent } from '@/types';
import { registerMmlLanguage } from './mmlLanguage';
import { registerMmlTheme } from './mmlTheme';

interface MonacoEditorProps {
  document: Document;
  onChange: (content: string) => void;
  settings: EditorSettings;
  /** The active document's language/driver id (e.g. 'gwi', 'muc', 'mus') */
  driverId?: string;
  // Trace playback props
  currentPosition?: Position | null;
  // Navigation
  navigationPosition?: Position | null;
  // Active note events from source map
  activeNoteEvents?: SourceMapEvent[];
}

// Expose editor methods to parent components
export interface MonacoEditorHandle {
  getEditor: () => any | null;
  triggerCommand: (command: string) => void;
  focus: () => void;
  hasSelection: () => boolean;
  canUndo: () => boolean;
  canRedo: () => boolean;
}

const MonacoEditor = forwardRef<MonacoEditorHandle, MonacoEditorProps>((
  { document, onChange, settings, driverId, currentPosition, navigationPosition, activeNoteEvents },
  ref
) => {
  const editorRef = useRef<any>(null);
  const monaco = useMonaco();
  const languageId = 'mml';
  // Decoration ID arrays are stored in refs, not state — they're internal
  // bookkeeping for Monaco, not anything React needs to re-render on. Storing
  // them as state created a `setX` → effect-deps → `setX` infinite loop.
  const currentLineDecorationsRef = useRef<string[]>([]);
  const navDecorationsRef = useRef<string[]>([]);
  const noteEventDecorationsRef = useRef<string[]>([]);
  // Use a ref so that the Monaco completion provider closure always reads the latest value
  const driverIdRef = useRef<string>(driverId ?? document.language ?? 'gwi');

  // Expose editor methods via ref
  useImperativeHandle(ref, () => ({
    getEditor: () => editorRef.current,
    triggerCommand: (command: string) => {
      if (editorRef.current) {
        editorRef.current.trigger('menu', command, null);
      }
    },
    focus: () => {
      editorRef.current?.focus();
    },
    hasSelection: () => {
      if (!editorRef.current) return false;
      const selection = editorRef.current.getSelection();
      return selection ? !selection.isEmpty() : false;
    },
    canUndo: () => {
      if (!editorRef.current) return false;
      return editorRef.current.getModel()?.canUndo() || false;
    },
    canRedo: () => {
      if (!editorRef.current) return false;
      return editorRef.current.getModel()?.canRedo() || false;
    },
  }));

  // Keep driverIdRef in sync with the prop so the completion provider
  // always reads the latest value without needing re-registration.
  useEffect(() => {
    driverIdRef.current = driverId ?? document.language ?? 'gwi';
  }, [driverId, document.language]);

  // Register MML language and theme when Monaco is loaded
  useEffect(() => {
    if (!monaco) return;

    // Register MML language, passing a stable callback that reads the ref.
    registerMmlLanguage(monaco, () => driverIdRef.current);

    // Register themes
    registerMmlTheme(monaco, 'mml-dark', 'vs-dark');
    registerMmlTheme(monaco, 'mml-light', 'vs');

    // Set the current theme
    monaco.editor.setTheme(settings.theme === 'vs-dark' ? 'mml-dark' : 'mml-light');
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [monaco, settings.theme]);

  // Highlight current playback position
  useEffect(() => {
    if (!editorRef.current || !currentPosition) return;

    const editor = editorRef.current;
    const monacoInstance = monaco;
    if (!monacoInstance) return;

    // Highlight current line (deltaDecorations replaces previous IDs atomically).
    const lineNumber = currentPosition.line;
    currentLineDecorationsRef.current = editor.deltaDecorations(currentLineDecorationsRef.current, [
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

    // Scroll to current line (if auto-scroll is enabled)
    const lineHeight = editor.getOption(monacoInstance.editor.EditorOption.lineHeight);
    const scrollTop = (lineNumber - 1) * lineHeight - editor.getScrollTop();
    const clientHeight = editor.getLayoutInfo().clientHeight;
    
    // Only scroll if line is not visible
    if (scrollTop > clientHeight || scrollTop < 0) {
      editor.revealLineInCenter(lineNumber);
    }
  }, [currentPosition, monaco]);

  // Handle navigation to a specific position (from error list click)
  useEffect(() => {
    if (!editorRef.current || !navigationPosition) return;

    const editor = editorRef.current;
    const monacoInstance = monaco;
    if (!monacoInstance) return;

    // Set cursor position and reveal line.
    const { line, column } = navigationPosition;
    editor.setPosition({ lineNumber: line, column: column });
    editor.revealLineInCenter(line);
    editor.focus();

    // Add temporary highlight (deltaDecorations clears the previous batch).
    navDecorationsRef.current = editor.deltaDecorations(navDecorationsRef.current, [
      {
        range: new monacoInstance.Range(line, column, line, column + 1),
        options: {
          className: 'navigate-highlight',
          isWholeLine: false,
        },
      },
    ]);

    const timer = setTimeout(() => {
      if (navDecorationsRef.current.length > 0) {
        editor.deltaDecorations(navDecorationsRef.current, []);
        navDecorationsRef.current = [];
      }
    }, 1000);

    return () => clearTimeout(timer);
  }, [navigationPosition, monaco]);

  // Highlight active note events from source map
  useEffect(() => {
    if (!editorRef.current || !activeNoteEvents || activeNoteEvents.length === 0) {
      if (noteEventDecorationsRef.current.length > 0) {
        editorRef.current?.deltaDecorations(noteEventDecorationsRef.current, []);
        noteEventDecorationsRef.current = [];
      }
      return;
    }

    const editor = editorRef.current;
    const monacoInstance = monaco;
    if (!monacoInstance) return;

    const newDecorations = activeNoteEvents.map((event) => ({
      range: new monacoInstance.Range(
        event.line,
        event.col_start,
        event.line,
        event.col_end
      ),
      options: {
        className: 'active-note',
        glyphMarginClassName: 'active-note-glyph',
        glyphMarginHoverMessage: {
          value: `Note: ${event.note_midi}, Part: ${event.part}, Instrument: ${event.instrument}`,
        },
        minimap: {
          color: '#FFD700',
          rasterized: true,
        },
      },
    }));

    noteEventDecorationsRef.current = editor.deltaDecorations(noteEventDecorationsRef.current, newDecorations);
  }, [activeNoteEvents, monaco]);

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
});

MonacoEditor.displayName = 'MonacoEditor';

export default MonacoEditor;
