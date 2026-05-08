import React, { useState, useEffect } from 'react';
import Modal from '@/components/Modal';
import { midiService } from '@/services/midiService';

interface MidiSettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

const selectStyle: React.CSSProperties = {
  width: '100%',
  padding: '5px 8px',
  background: 'var(--bg-tertiary)',
  border: '1px solid var(--border-color)',
  borderRadius: '4px',
  color: 'var(--fg-primary)',
  fontSize: '13px',
};

const labelStyle: React.CSSProperties = {
  display: 'block',
  marginBottom: '4px',
  fontSize: '12px',
  color: 'var(--fg-muted)',
};

const rowStyle: React.CSSProperties = { marginBottom: '14px' };

const MidiSettingsDialog: React.FC<MidiSettingsDialogProps> = ({ isOpen, onClose }) => {
  const [supported, setSupported] = useState(false);
  const [enabled, setEnabled] = useState(false);
  const [mode, setMode] = useState<'preview' | 'input'>('preview');
  const [devices, setDevices] = useState<{ id: string; name: string }[]>([]);

  useEffect(() => {
    if (!isOpen) return;
    setSupported(midiService.isSupported());
    setEnabled(midiService.isEnabled());
    setMode(midiService.getMode());
    const inputs = midiService.getInputDevices();
    setDevices(inputs.map((d) => ({ id: d.id, name: d.name })));
  }, [isOpen]);

  if (!supported) {
    return (
      <Modal
        isOpen={isOpen}
        onClose={onClose}
        title="MIDI Settings"
        footer={<button className="button primary" onClick={onClose}>Close</button>}
      >
        <p style={{ color: 'var(--fg-muted)', fontSize: '13px' }}>
          Web MIDI API is not available in this browser. Try Chrome or Edge with a MIDI device connected.
        </p>
      </Modal>
    );
  }

  const handleSave = () => {
    if (enabled) {
      midiService.enable();
    } else {
      midiService.disable();
    }
    midiService.setMode(mode);
    onClose();
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="MIDI Settings"
      width={400}
      footer={
        <>
          <button className="button" onClick={onClose}>Cancel</button>
          <button className="button primary" onClick={handleSave}>Save</button>
        </>
      }
    >
      <div style={rowStyle}>
        <label style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '13px', cursor: 'pointer' }}>
          <input
            type="checkbox"
            checked={enabled}
            onChange={(e) => setEnabled(e.target.checked)}
          />
          Enable MIDI input
        </label>
      </div>

      <div style={rowStyle}>
        <label style={labelStyle}>Mode</label>
        <select
          style={selectStyle}
          value={mode}
          onChange={(e) => setMode(e.target.value as 'preview' | 'input')}
          disabled={!enabled}
        >
          <option value="preview">Preview (play notes in chip emulator)</option>
          <option value="input">Input (record to editor)</option>
        </select>
      </div>

      <div style={rowStyle}>
        <label style={labelStyle}>Detected Input Devices</label>
        {devices.length === 0 ? (
          <p style={{ fontSize: '12px', color: 'var(--fg-muted)', margin: 0 }}>
            No MIDI devices detected. Connect a device and reopen this dialog.
          </p>
        ) : (
          <ul style={{ margin: 0, padding: '0 0 0 16px', fontSize: '12px' }}>
            {devices.map((d) => <li key={d.id}>{d.name}</li>)}
          </ul>
        )}
      </div>
    </Modal>
  );
};

export default MidiSettingsDialog;
