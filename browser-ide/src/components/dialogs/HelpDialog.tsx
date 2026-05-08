import React, { useState } from 'react';
import Modal from '@/components/Modal';

interface HelpSection {
  title: string;
  content: React.ReactNode;
}

const SECTIONS: HelpSection[] = [
  {
    title: 'Getting Started',
    content: (
      <div>
        <p>Welcome to the mml2vgm Browser IDE. Here's how to get started:</p>
        <ol style={{ paddingLeft: '20px', lineHeight: 1.8 }}>
          <li>Open the <strong>Examples</strong> menu to load a sample song.</li>
          <li>Press <kbd>F5</kbd> or click <strong>Compile &amp; Play</strong> to hear it.</li>
          <li>Edit the MML text in the editor to make changes.</li>
          <li>Use <strong>File → Save As…</strong> to save your work.</li>
          <li>Use <strong>File → Export…</strong> to download the compiled VGM file.</li>
        </ol>
      </div>
    ),
  },
  {
    title: 'MML Syntax Overview',
    content: (
      <div>
        <p>MML (Music Macro Language) uses single-letter commands to describe music:</p>
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '12px' }}>
          <tbody>
            {[
              ['A–G', 'Play a note (A, B, C, D, E, F, G)'],
              ['+ / -', 'Sharp / flat modifier after a note'],
              ['R', 'Rest'],
              ['O<n>', 'Set octave (O4 = middle octave)'],
              ['< / >', 'Octave down / up'],
              ['L<n>', 'Set default note length (L4 = quarter note)'],
              ['T<n>', 'Set tempo in BPM'],
              ['V<n>', 'Set volume (0–15)'],
              ['@<n>', 'Set instrument/patch number'],
              ['[ … ]<n>', 'Repeat block n times'],
            ].map(([cmd, desc]) => (
              <tr key={cmd} style={{ borderBottom: '1px solid var(--border-color, rgba(128,128,128,0.15))' }}>
                <td style={{ padding: '4px 8px', fontFamily: 'monospace', width: '120px' }}>{cmd}</td>
                <td style={{ padding: '4px 8px', color: 'var(--fg-muted)' }}>{desc}</td>
              </tr>
            ))}
          </tbody>
        </table>
        <p style={{ marginTop: '8px', fontSize: '12px', color: 'var(--fg-muted)' }}>
          Use <strong>Help → MML Reference</strong> for a full command reference.
        </p>
      </div>
    ),
  },
  {
    title: 'Keyboard Shortcuts',
    content: (
      <p>
        Open <strong>Help → Keyboard Shortcuts</strong> for the complete list of shortcuts.
        Key shortcuts include <kbd>F5</kbd> (Compile &amp; Play), <kbd>F7</kbd> (Compile),
        <kbd>Ctrl+S</kbd> (Save), <kbd>Ctrl+F</kbd> (Find), and <kbd>Ctrl+Z</kbd> (Undo).
      </p>
    ),
  },
  {
    title: 'Browser Compatibility',
    content: (
      <div>
        <p>The IDE runs best in <strong>Chrome</strong> or <strong>Edge</strong>.</p>
        <ul style={{ paddingLeft: '20px', lineHeight: 1.8 }}>
          <li><strong>File System Access API</strong> (Save/Open) — Chrome/Edge only; Firefox uses download fallback.</li>
          <li><strong>Web MIDI</strong> — Chrome/Edge only.</li>
          <li><strong>AudioWorklet</strong> — All modern browsers (Firefox 76+, Safari 14.1+).</li>
          <li><strong>SharedArrayBuffer</strong> — Requires COOP/COEP headers; the IDE's service worker sets these.</li>
        </ul>
      </div>
    ),
  },
  {
    title: 'Samples and PCM',
    content: (
      <p>
        Upload WAV or OGG samples via the <strong>Samples</strong> panel in the bottom bar.
        Reference them in MML using <code>@SAMPLE(&lt;filename&gt;)</code>. Samples are stored
        per-document in the browser's IndexedDB.
      </p>
    ),
  },
];

const kbdStyle: React.CSSProperties = {
  display: 'inline-block',
  padding: '1px 6px',
  background: 'var(--bg-tertiary)',
  border: '1px solid var(--border-color)',
  borderRadius: '3px',
  fontSize: '11px',
  fontFamily: 'monospace',
};

interface HelpDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

const HelpDialog: React.FC<HelpDialogProps> = ({ isOpen, onClose }) => {
  const [openSection, setOpenSection] = useState<number | null>(0);

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Help Topics"
      width={560}
      footer={<button className="button primary" onClick={onClose}>Close</button>}
    >
      <style>{`kbd { ${Object.entries(kbdStyle).map(([k, v]) => `${k.replace(/([A-Z])/g, '-$1').toLowerCase()}:${v}`).join(';')} }`}</style>
      <div style={{ maxHeight: '400px', overflowY: 'auto' }}>
        {SECTIONS.map((section, i) => (
          <div key={i} style={{ borderBottom: '1px solid var(--border-color)' }}>
            <button
              onClick={() => setOpenSection(openSection === i ? null : i)}
              style={{
                width: '100%',
                textAlign: 'left',
                padding: '10px 8px',
                background: 'none',
                border: 'none',
                cursor: 'pointer',
                color: 'var(--fg-primary)',
                fontSize: '13px',
                fontWeight: 600,
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
              }}
            >
              {section.title}
              <span style={{ fontSize: '10px', color: 'var(--fg-muted)' }}>
                {openSection === i ? '▲' : '▼'}
              </span>
            </button>
            {openSection === i && (
              <div
                style={{
                  padding: '0 8px 12px',
                  fontSize: '13px',
                  lineHeight: 1.6,
                  color: 'var(--fg-secondary, var(--fg-primary))',
                }}
              >
                {section.content}
              </div>
            )}
          </div>
        ))}
      </div>
    </Modal>
  );
};

export default HelpDialog;
