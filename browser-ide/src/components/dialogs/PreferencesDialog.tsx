import React, { useState } from 'react';
import Modal from '@/components/Modal';
import type { IDESettings, EditorSettings, AudioSettings } from '@/types';

interface PreferencesDialogProps {
  isOpen: boolean;
  onClose: () => void;
  settings: IDESettings;
  onSave: (updates: Partial<IDESettings>) => void;
}

type Tab = 'editor' | 'compiler' | 'playback' | 'appearance';

const TAB_LABELS: Record<Tab, string> = {
  editor: 'Editor',
  compiler: 'Compiler',
  playback: 'Playback',
  appearance: 'Appearance',
};

const labelStyle: React.CSSProperties = {
  display: 'block',
  marginBottom: '4px',
  fontSize: '12px',
  color: 'var(--fg-muted)',
};

const rowStyle: React.CSSProperties = { marginBottom: '14px' };

const inputStyle: React.CSSProperties = {
  width: '100%',
  boxSizing: 'border-box',
  padding: '5px 8px',
  background: 'var(--bg-tertiary)',
  border: '1px solid var(--border-color)',
  borderRadius: '4px',
  color: 'var(--fg-primary)',
  fontSize: '13px',
};

const selectStyle = inputStyle;

const checkboxRow = (
  label: string,
  checked: boolean,
  onChange: (v: boolean) => void
) => (
  <div style={rowStyle}>
    <label style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '13px', cursor: 'pointer' }}>
      <input type="checkbox" checked={checked} onChange={(e) => onChange(e.target.checked)} />
      {label}
    </label>
  </div>
);

const PreferencesDialog: React.FC<PreferencesDialogProps> = ({
  isOpen,
  onClose,
  settings,
  onSave,
}) => {
  const [tab, setTab] = useState<Tab>('editor');
  const [localEditor, setLocalEditor] = useState<Partial<EditorSettings>>({});
  const [localAudio, setLocalAudio] = useState<Partial<AudioSettings>>({});
  const [localMisc, setLocalMisc] = useState<Partial<IDESettings>>({});

  const editor = { ...settings.editor, ...localEditor };
  const audio = { ...settings.audio, ...localAudio };
  const misc = { ...settings, ...localMisc };

  const setEditor = <K extends keyof EditorSettings>(key: K, val: EditorSettings[K]) =>
    setLocalEditor((p) => ({ ...p, [key]: val }));
  const setAudio = <K extends keyof AudioSettings>(key: K, val: AudioSettings[K]) =>
    setLocalAudio((p) => ({ ...p, [key]: val }));
  const setMisc = <K extends keyof IDESettings>(key: K, val: IDESettings[K]) =>
    setLocalMisc((p) => ({ ...p, [key]: val }));

  const handleSave = () => {
    const updates: Partial<IDESettings> = { ...localMisc };
    if (Object.keys(localEditor).length) updates.editor = { ...settings.editor, ...localEditor };
    if (Object.keys(localAudio).length) updates.audio = { ...settings.audio, ...localAudio };
    onSave(updates);
    setLocalEditor({});
    setLocalAudio({});
    setLocalMisc({});
    onClose();
  };

  const handleCancel = () => {
    setLocalEditor({});
    setLocalAudio({});
    setLocalMisc({});
    onClose();
  };

  const tabBtnStyle = (t: Tab): React.CSSProperties => ({
    padding: '6px 14px',
    fontSize: '13px',
    background: tab === t ? 'var(--accent-primary, #4a90d9)' : 'var(--bg-tertiary)',
    color: tab === t ? '#fff' : 'var(--fg-primary)',
    border: '1px solid var(--border-color)',
    borderRadius: '4px',
    cursor: 'pointer',
  });

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleCancel}
      title="Preferences"
      width={520}
      footer={
        <>
          <button className="button" onClick={handleCancel}>Cancel</button>
          <button className="button primary" onClick={handleSave}>Save</button>
        </>
      }
    >
      {/* Tab bar */}
      <div style={{ display: 'flex', gap: '6px', marginBottom: '16px', flexWrap: 'wrap' }}>
        {(Object.keys(TAB_LABELS) as Tab[]).map((t) => (
          <button key={t} style={tabBtnStyle(t)} onClick={() => setTab(t)}>
            {TAB_LABELS[t]}
          </button>
        ))}
      </div>

      {/* Editor tab */}
      {tab === 'editor' && (
        <>
          <div style={rowStyle}>
            <label style={labelStyle}>Font Size ({editor.fontSize}px)</label>
            <input
              type="range" min={8} max={32} value={editor.fontSize}
              onChange={(e) => setEditor('fontSize', Number(e.target.value))}
              style={{ width: '100%' }}
            />
          </div>
          <div style={rowStyle}>
            <label style={labelStyle}>Tab Size</label>
            <select style={selectStyle} value={editor.tabSize} onChange={(e) => setEditor('tabSize', Number(e.target.value))}>
              {[2, 4, 8].map((n) => <option key={n} value={n}>{n} spaces</option>)}
            </select>
          </div>
          <div style={rowStyle}>
            <label style={labelStyle}>Theme</label>
            <select style={selectStyle} value={editor.theme} onChange={(e) => setEditor('theme', e.target.value as EditorSettings['theme'])}>
              <option value="vs-dark">Dark</option>
              <option value="vs">Light</option>
              <option value="hc-black">High Contrast</option>
            </select>
          </div>
          {checkboxRow('Word Wrap', editor.wordWrap, (v) => setEditor('wordWrap', v))}
          {checkboxRow('Show Line Numbers', editor.showLineNumbers, (v) => setEditor('showLineNumbers', v))}
          {checkboxRow('Show Minimap', editor.showMinimap, (v) => setEditor('showMinimap', v))}
        </>
      )}

      {/* Compiler tab */}
      {tab === 'compiler' && (
        <>
          <div style={rowStyle}>
            <label style={labelStyle}>Default Output Format</label>
            <select
              style={selectStyle}
              value={misc.outputFormat}
              onChange={(e) => setMisc('outputFormat', e.target.value as IDESettings['outputFormat'])}
            >
              <option value="vgm">VGM</option>
              <option value="xgm">XGM</option>
              <option value="xgm2">XGM2</option>
              <option value="zgm">ZGM</option>
            </select>
          </div>
          {checkboxRow('Auto-save before compile', misc.autoSave ?? true, (v) => setMisc('autoSave', v))}
        </>
      )}

      {/* Playback tab */}
      {tab === 'playback' && (
        <>
          <div style={rowStyle}>
            <label style={labelStyle}>Master Volume ({audio.masterVolume ?? 100}%)</label>
            <input
              type="range" min={0} max={100}
              value={audio.masterVolume ?? 100}
              onChange={(e) => setAudio('masterVolume', Number(e.target.value))}
              style={{ width: '100%' }}
            />
          </div>
          <div style={rowStyle}>
            <label style={labelStyle}>Default Playback Rate</label>
            <select
              style={selectStyle}
              value={audio.playbackRate ?? 1.0}
              onChange={(e) => setAudio('playbackRate', Number(e.target.value))}
            >
              {[0.5, 0.75, 1.0, 1.25, 1.5, 2.0].map((r) => (
                <option key={r} value={r}>{Math.round(r * 100)}%</option>
              ))}
            </select>
          </div>
          {checkboxRow('Loop by default', audio.loop ?? false, (v) => setAudio('loop', v))}
        </>
      )}

      {/* Appearance tab */}
      {tab === 'appearance' && (
        <>
          <div style={rowStyle}>
            <label style={labelStyle}>Application Theme</label>
            <select
              style={selectStyle}
              value={misc.theme}
              onChange={(e) => setMisc('theme', e.target.value as IDESettings['theme'])}
            >
              <option value="dark">Dark</option>
              <option value="light">Light</option>
              <option value="system">System</option>
            </select>
          </div>
          <div style={rowStyle}>
            <label style={labelStyle}>Language</label>
            <select
              style={selectStyle}
              value={misc.language}
              onChange={(e) => setMisc('language', e.target.value)}
            >
              <option value="en">English</option>
              <option value="ja">日本語</option>
            </select>
          </div>
        </>
      )}
    </Modal>
  );
};

export default PreferencesDialog;
