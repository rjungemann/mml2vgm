import React, { useState, useEffect } from 'react';
import Modal from '@/components/Modal';
import type { ChipInfo } from '@/types';

export interface AdvancedCompileOptions {
  targetChips: string[];
  gd3Title: string;
  gd3Game: string;
  gd3Author: string;
  gd3Date: string;
  strictMode: boolean;
}

interface AdvancedCompileOptionsDialogProps {
  isOpen: boolean;
  onClose: () => void;
  chips: ChipInfo[];
  options: AdvancedCompileOptions;
  onSave: (options: AdvancedCompileOptions) => void;
}

const labelStyle: React.CSSProperties = {
  display: 'block',
  marginBottom: '4px',
  fontSize: '12px',
  color: 'var(--fg-muted)',
};

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

const rowStyle: React.CSSProperties = { marginBottom: '12px' };

const AdvancedCompileOptionsDialog: React.FC<AdvancedCompileOptionsDialogProps> = ({
  isOpen,
  onClose,
  chips,
  options,
  onSave,
}) => {
  const [local, setLocal] = useState<AdvancedCompileOptions>(options);

  useEffect(() => {
    if (isOpen) setLocal(options);
  }, [isOpen, options]);

  const toggleChip = (variant: string) => {
    setLocal((prev) => ({
      ...prev,
      targetChips: prev.targetChips.includes(variant)
        ? prev.targetChips.filter((c) => c !== variant)
        : [...prev.targetChips, variant],
    }));
  };

  const handleSave = () => {
    onSave(local);
    onClose();
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Advanced Compile Options"
      width={520}
      footer={
        <>
          <button className="button" onClick={onClose}>
            Cancel
          </button>
          <button className="button primary" onClick={handleSave}>
            Apply
          </button>
        </>
      }
    >
      <div style={rowStyle}>
        <label style={labelStyle}>Target Chips</label>
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(3, 1fr)',
            gap: '4px',
            maxHeight: '150px',
            overflowY: 'auto',
            border: '1px solid var(--border-color)',
            borderRadius: '4px',
            padding: '6px',
          }}
        >
          {chips.length === 0 && (
            <span style={{ color: 'var(--fg-muted)', fontSize: '12px', gridColumn: '1/-1' }}>
              Loading chips…
            </span>
          )}
          {chips.map((chip) => (
            <label
              key={chip.variant}
              style={{ display: 'flex', alignItems: 'center', gap: '5px', fontSize: '12px', cursor: 'pointer' }}
            >
              <input
                type="checkbox"
                checked={local.targetChips.includes(chip.variant.toLowerCase())}
                onChange={() => toggleChip(chip.variant.toLowerCase())}
              />
              {chip.name}
            </label>
          ))}
        </div>
      </div>

      <div style={{ borderTop: '1px solid var(--border-color)', paddingTop: '12px', marginBottom: '12px' }}>
        <label style={{ ...labelStyle, marginBottom: '8px', fontWeight: 600 }}>GD3 Tags</label>
        {(
          [
            { key: 'gd3Title', label: 'Track Title' },
            { key: 'gd3Game', label: 'Game Name' },
            { key: 'gd3Author', label: 'Author' },
            { key: 'gd3Date', label: 'Release Date' },
          ] as const
        ).map(({ key, label }) => (
          <div key={key} style={rowStyle}>
            <label style={labelStyle}>{label}</label>
            <input
              type="text"
              style={inputStyle}
              value={local[key]}
              onChange={(e) => setLocal((prev) => ({ ...prev, [key]: e.target.value }))}
            />
          </div>
        ))}
      </div>

      <label style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '13px', cursor: 'pointer' }}>
        <input
          type="checkbox"
          checked={local.strictMode}
          onChange={(e) => setLocal((prev) => ({ ...prev, strictMode: e.target.checked }))}
        />
        Strict mode (warnings as errors)
      </label>
    </Modal>
  );
};

export default AdvancedCompileOptionsDialog;
