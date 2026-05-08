import React, { useState } from 'react';
import Modal from '@/components/Modal';

interface RefCategory {
  name: string;
  commands: Array<{ syntax: string; description: string; example?: string }>;
}

const REFERENCE: RefCategory[] = [
  {
    name: 'Notes & Rests',
    commands: [
      { syntax: 'A–G', description: 'Play note A through G at current octave and default length.' },
      { syntax: 'A+ / A-', description: 'Sharp (+) or flat (-) modifier immediately after a note letter.' },
      { syntax: 'A<n>', description: 'Play note with explicit length: 1=whole, 2=half, 4=quarter, 8=eighth…', example: 'C4 D8 E8 F2' },
      { syntax: 'A<n>.', description: 'Dotted note — 1.5× the base length.', example: 'C4.' },
      { syntax: 'R', description: 'Rest for the default length.', example: 'C4 R4 D4' },
      { syntax: 'R<n>', description: 'Rest for an explicit length.', example: 'C4 R8 E4' },
    ],
  },
  {
    name: 'Octave & Pitch',
    commands: [
      { syntax: 'O<n>', description: 'Set octave (usually 0–8). O4 is middle octave.', example: 'O4 C' },
      { syntax: '<', description: 'Shift octave down by one.', example: 'O5 C < C' },
      { syntax: '>', description: 'Shift octave up by one.', example: 'O4 C > C' },
    ],
  },
  {
    name: 'Duration & Tempo',
    commands: [
      { syntax: 'L<n>', description: 'Set default note length. Subsequent notes without a length use this.', example: 'L8 C D E F' },
      { syntax: 'T<n>', description: 'Set tempo in BPM.', example: 'T120' },
    ],
  },
  {
    name: 'Volume & Expression',
    commands: [
      { syntax: 'V<n>', description: 'Set volume (range varies by chip; typically 0–15).', example: 'V12 C4' },
      { syntax: 'Q<n>', description: 'Set quantize — note gate time as fraction of length (1–8).', example: 'Q6 C4' },
    ],
  },
  {
    name: 'Instrument & Patches',
    commands: [
      { syntax: '@<n>', description: 'Select instrument or patch number.', example: '@1 C4' },
      { syntax: '@SAMPLE(<file>)', description: 'Reference an uploaded PCM sample by filename.', example: '@SAMPLE(kick.wav) C4' },
    ],
  },
  {
    name: 'Control Flow',
    commands: [
      { syntax: '[ … ]<n>', description: 'Repeat the enclosed commands n times.', example: '[C4 E4 G4]2' },
      { syntax: ';', description: 'End of track / comment line.', example: '; this is a comment' },
    ],
  },
  {
    name: 'Parts',
    commands: [
      { syntax: 'PART <name>', description: 'Begin a new part (channel/voice) assignment.' },
      { syntax: 'CH <chip> <channel>', description: 'Assign the current part to a specific chip and channel.' },
    ],
  },
];

interface MmlReferenceDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

const MmlReferenceDialog: React.FC<MmlReferenceDialogProps> = ({ isOpen, onClose }) => {
  const [selectedCategory, setSelectedCategory] = useState(0);
  const [search, setSearch] = useState('');

  const category = REFERENCE[selectedCategory];
  const commands = search
    ? category.commands.filter(
        (c) =>
          c.syntax.toLowerCase().includes(search.toLowerCase()) ||
          c.description.toLowerCase().includes(search.toLowerCase())
      )
    : category.commands;

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="MML Reference"
      width={640}
      footer={<button className="button primary" onClick={onClose}>Close</button>}
    >
      <div style={{ display: 'flex', gap: '12px', height: '420px' }}>
        {/* Category list */}
        <div
          style={{
            width: '160px',
            flexShrink: 0,
            borderRight: '1px solid var(--border-color)',
            paddingRight: '8px',
            overflowY: 'auto',
          }}
        >
          {REFERENCE.map((cat, i) => (
            <button
              key={i}
              onClick={() => { setSelectedCategory(i); setSearch(''); }}
              style={{
                display: 'block',
                width: '100%',
                textAlign: 'left',
                padding: '7px 8px',
                background: selectedCategory === i ? 'var(--accent-primary, #4a90d9)' : 'none',
                color: selectedCategory === i ? '#fff' : 'var(--fg-primary)',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '12px',
                marginBottom: '2px',
              }}
            >
              {cat.name}
            </button>
          ))}
        </div>

        {/* Content */}
        <div style={{ flex: 1, overflowY: 'auto' }}>
          <input
            type="search"
            placeholder="Filter…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            style={{
              width: '100%',
              boxSizing: 'border-box',
              padding: '5px 8px',
              marginBottom: '10px',
              background: 'var(--bg-tertiary)',
              border: '1px solid var(--border-color)',
              borderRadius: '4px',
              color: 'var(--fg-primary)',
              fontSize: '12px',
            }}
          />
          {commands.map((cmd, i) => (
            <div
              key={i}
              style={{
                borderBottom: '1px solid var(--border-color, rgba(128,128,128,0.15))',
                paddingBottom: '10px',
                marginBottom: '10px',
              }}
            >
              <code
                style={{
                  display: 'block',
                  fontSize: '13px',
                  background: 'var(--bg-tertiary)',
                  padding: '3px 6px',
                  borderRadius: '3px',
                  marginBottom: '4px',
                  width: 'fit-content',
                }}
              >
                {cmd.syntax}
              </code>
              <p style={{ margin: '0 0 4px', fontSize: '12px', lineHeight: 1.5 }}>{cmd.description}</p>
              {cmd.example && (
                <code style={{ fontSize: '11px', color: 'var(--fg-muted)' }}>
                  Example: {cmd.example}
                </code>
              )}
            </div>
          ))}
          {commands.length === 0 && (
            <p style={{ color: 'var(--fg-muted)', fontSize: '13px' }}>No commands match "{search}"</p>
          )}
        </div>
      </div>
    </Modal>
  );
};

export default MmlReferenceDialog;
