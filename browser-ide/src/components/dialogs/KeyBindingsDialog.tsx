import React, { useState } from 'react';
import Modal from '@/components/Modal';

interface KeyBinding {
  action: string;
  shortcut: string;
  context: 'global' | 'editor' | 'playback';
}

const KEYBINDINGS: KeyBinding[] = [
  // Compile / Play
  { action: 'Compile & Play', shortcut: 'F5', context: 'global' },
  { action: 'Compile', shortcut: 'F7', context: 'global' },
  // File
  { action: 'New Document', shortcut: 'Ctrl+N', context: 'global' },
  { action: 'Open File', shortcut: 'Ctrl+O', context: 'global' },
  { action: 'Save', shortcut: 'Ctrl+S', context: 'global' },
  { action: 'Save As', shortcut: 'Ctrl+Shift+S', context: 'global' },
  // Edit
  { action: 'Undo', shortcut: 'Ctrl+Z', context: 'editor' },
  { action: 'Redo', shortcut: 'Ctrl+Y / Ctrl+Shift+Z', context: 'editor' },
  { action: 'Cut', shortcut: 'Ctrl+X', context: 'editor' },
  { action: 'Copy', shortcut: 'Ctrl+C', context: 'editor' },
  { action: 'Paste', shortcut: 'Ctrl+V', context: 'editor' },
  { action: 'Select All', shortcut: 'Ctrl+A', context: 'editor' },
  { action: 'Find', shortcut: 'Ctrl+F', context: 'editor' },
  { action: 'Replace', shortcut: 'Ctrl+H', context: 'editor' },
  { action: 'Toggle Comment', shortcut: 'Ctrl+/', context: 'editor' },
  { action: 'Format Document', shortcut: 'Shift+Alt+F', context: 'editor' },
  { action: 'Go to Line', shortcut: 'Ctrl+G', context: 'editor' },
  // View
  { action: 'Zoom In', shortcut: 'Ctrl+=', context: 'global' },
  { action: 'Zoom Out', shortcut: 'Ctrl+-', context: 'global' },
  { action: 'Reset Zoom', shortcut: 'Ctrl+0', context: 'global' },
  // Help
  { action: 'Help Topics', shortcut: 'F1', context: 'global' },
  { action: 'Key Bindings', shortcut: 'Ctrl+K Ctrl+S', context: 'global' },
];

const CONTEXT_LABELS: Record<KeyBinding['context'], string> = {
  global: 'Global',
  editor: 'Editor',
  playback: 'Playback',
};

const CONTEXT_COLORS: Record<KeyBinding['context'], string> = {
  global: 'var(--accent-primary, #4a90d9)',
  editor: 'var(--status-info-fg, #4a90d9)',
  playback: 'var(--status-success-fg, #4caf50)',
};

interface KeyBindingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

const KeyBindingsDialog: React.FC<KeyBindingsDialogProps> = ({ isOpen, onClose }) => {
  const [search, setSearch] = useState('');

  const filtered = search
    ? KEYBINDINGS.filter(
        (kb) =>
          kb.action.toLowerCase().includes(search.toLowerCase()) ||
          kb.shortcut.toLowerCase().includes(search.toLowerCase())
      )
    : KEYBINDINGS;

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Keyboard Shortcuts"
      width={560}
      footer={
        <button className="button primary" onClick={onClose}>
          Close
        </button>
      }
    >
      <input
        type="search"
        placeholder="Filter shortcuts…"
        value={search}
        onChange={(e) => setSearch(e.target.value)}
        style={{
          width: '100%',
          boxSizing: 'border-box',
          padding: '6px 8px',
          marginBottom: '12px',
          background: 'var(--bg-tertiary)',
          border: '1px solid var(--border-color)',
          borderRadius: '4px',
          color: 'var(--fg-primary)',
          fontSize: '13px',
        }}
      />
      <div style={{ maxHeight: '380px', overflowY: 'auto' }}>
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '13px' }}>
          <thead>
            <tr style={{ borderBottom: '1px solid var(--border-color)' }}>
              <th style={{ textAlign: 'left', padding: '4px 8px', color: 'var(--fg-muted)' }}>Action</th>
              <th style={{ textAlign: 'left', padding: '4px 8px', color: 'var(--fg-muted)' }}>Shortcut</th>
              <th style={{ textAlign: 'left', padding: '4px 8px', color: 'var(--fg-muted)' }}>Context</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((kb, i) => (
              <tr
                key={i}
                style={{
                  borderBottom: '1px solid var(--border-color, rgba(128,128,128,0.15))',
                }}
              >
                <td style={{ padding: '5px 8px' }}>{kb.action}</td>
                <td style={{ padding: '5px 8px' }}>
                  <code
                    style={{
                      background: 'var(--bg-tertiary)',
                      border: '1px solid var(--border-color)',
                      borderRadius: '3px',
                      padding: '1px 5px',
                      fontSize: '12px',
                    }}
                  >
                    {kb.shortcut}
                  </code>
                </td>
                <td style={{ padding: '5px 8px' }}>
                  <span
                    style={{
                      fontSize: '11px',
                      color: CONTEXT_COLORS[kb.context],
                      border: `1px solid ${CONTEXT_COLORS[kb.context]}`,
                      borderRadius: '3px',
                      padding: '1px 5px',
                    }}
                  >
                    {CONTEXT_LABELS[kb.context]}
                  </span>
                </td>
              </tr>
            ))}
            {filtered.length === 0 && (
              <tr>
                <td colSpan={3} style={{ padding: '16px', textAlign: 'center', color: 'var(--fg-muted)' }}>
                  No shortcuts match "{search}"
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </Modal>
  );
};

export default KeyBindingsDialog;
