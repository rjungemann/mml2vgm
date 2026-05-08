import React, { useState } from 'react';
import Modal from '@/components/Modal';
import type { AudioSettings } from '@/types';

interface AudioSettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
  settings: AudioSettings;
  onSave: (settings: Partial<AudioSettings>) => void;
}

const SAMPLE_RATES = [22050, 44100, 48000];
const BUFFER_SIZES = [512, 1024, 2048, 4096];

const labelStyle: React.CSSProperties = {
  display: 'block',
  marginBottom: '4px',
  fontSize: '12px',
  color: 'var(--fg-muted)',
};

const rowStyle: React.CSSProperties = {
  marginBottom: '14px',
};

const selectStyle: React.CSSProperties = {
  width: '100%',
  padding: '5px 8px',
  background: 'var(--bg-tertiary)',
  border: '1px solid var(--border-color)',
  borderRadius: '4px',
  color: 'var(--fg-primary)',
  fontSize: '13px',
};

const AudioSettingsDialog: React.FC<AudioSettingsDialogProps> = ({
  isOpen,
  onClose,
  settings,
  onSave,
}) => {
  const [local, setLocal] = useState<Partial<AudioSettings>>({});
  const current = { ...settings, ...local };

  const set = <K extends keyof AudioSettings>(key: K, value: AudioSettings[K]) =>
    setLocal((prev) => ({ ...prev, [key]: value }));

  const handleSave = () => {
    onSave(local);
    setLocal({});
    onClose();
  };

  const handleCancel = () => {
    setLocal({});
    onClose();
  };

  const bufferChanged =
    local.bufferSize !== undefined && local.bufferSize !== settings.bufferSize;
  const sampleRateChanged =
    local.sampleRate !== undefined && local.sampleRate !== settings.sampleRate;
  const requiresRestart = bufferChanged || sampleRateChanged;

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleCancel}
      title="Audio Settings"
      width={400}
      footer={
        <>
          <button className="button" onClick={handleCancel}>
            Cancel
          </button>
          <button className="button primary" onClick={handleSave}>
            Save
          </button>
        </>
      }
    >
      <div style={rowStyle}>
        <label style={labelStyle}>Master Volume ({current.masterVolume ?? 100}%)</label>
        <input
          type="range"
          min={0}
          max={100}
          value={current.masterVolume ?? 100}
          onChange={(e) => set('masterVolume', Number(e.target.value))}
          style={{ width: '100%' }}
        />
      </div>

      <div style={rowStyle}>
        <label style={labelStyle}>Sample Rate</label>
        <select
          style={selectStyle}
          value={current.sampleRate ?? 44100}
          onChange={(e) => set('sampleRate', Number(e.target.value))}
        >
          {SAMPLE_RATES.map((r) => (
            <option key={r} value={r}>
              {r.toLocaleString()} Hz
            </option>
          ))}
        </select>
      </div>

      <div style={rowStyle}>
        <label style={labelStyle}>Buffer Size</label>
        <select
          style={selectStyle}
          value={current.bufferSize ?? 4096}
          onChange={(e) => set('bufferSize', Number(e.target.value))}
        >
          {BUFFER_SIZES.map((b) => (
            <option key={b} value={b}>
              {b} samples
            </option>
          ))}
        </select>
      </div>

      {requiresRestart && (
        <div
          style={{
            padding: '8px 10px',
            background: 'var(--status-warning-bg, rgba(255,200,0,0.1))',
            border: '1px solid var(--status-warning-fg, #c8a000)',
            borderRadius: '4px',
            fontSize: '12px',
            color: 'var(--status-warning-fg, #c8a000)',
          }}
        >
          ⚠ Sample rate or buffer size changes take effect after restarting playback.
        </div>
      )}
    </Modal>
  );
};

export default AudioSettingsDialog;
