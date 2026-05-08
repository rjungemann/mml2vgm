/**
 * PMD Rhythm Editor Panel
 *
 * Provides a 16-step sequencer grid for the YM2608 rhythm instruments
 * used in the PC-98 PMD (Professional Music Driver) format.
 *
 * The grid lets the user toggle steps for each of the six rhythm channels
 * (BD, SD, TOM, HH, CYM, RIM) and generates / parses the @RHYTHM section.
 */

import React, { useState, useCallback } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { useDocumentStore } from '@/stores/documentStore';

// Rhythm channels in YM2608 order
const RHYTHM_CHANNELS = ['BD', 'SD', 'TOM', 'HH', 'CYM', 'RIM'] as const;
type RhythmChannel = typeof RHYTHM_CHANNELS[number];

const CHANNEL_LABELS: Record<RhythmChannel, string> = {
  BD:  'Bass Drum',
  SD:  'Snare',
  TOM: 'Tom',
  HH:  'Hi-hat',
  CYM: 'Cymbal',
  RIM: 'Rim Shot',
};

const STEPS = 16;

type Grid = Record<RhythmChannel, boolean[]>;

function emptyGrid(): Grid {
  const grid = {} as Grid;
  for (const ch of RHYTHM_CHANNELS) {
    grid[ch] = Array(STEPS).fill(false);
  }
  return grid;
}

/** Parse @RHYTHM block from document content.
 *  Looks for lines like:  BD  | x . . . x . . . | (x = on, . = off)
 *  and also understands bare comma/space separated hex pattern tokens.
 */
function parseRhythmSection(content: string): Grid {
  const grid = emptyGrid();
  const lines = content.split('\n');

  for (const line of lines) {
    const trimmed = line.trim();
    // Look for "CHANNEL  x . x . …" style
    const match = trimmed.match(/^(BD|SD|TOM|HH|CYM|RIM)\s+([\s\.\,xX01|]+)/i);
    if (match) {
      const ch = match[1].toUpperCase() as RhythmChannel;
      const patternStr = match[2];
      // Extract step tokens: x/X/1 = on, ./0 = off, | separators ignored
      const tokens = patternStr.split('').filter(c => /[xX01\.]/.test(c));
      tokens.slice(0, STEPS).forEach((t, i) => {
        grid[ch][i] = /[xX1]/.test(t);
      });
    }
  }
  return grid;
}

/** Serialise the grid to an @RHYTHM block string. */
function serializeGrid(grid: Grid): string {
  const lines = ['@RHYTHM'];
  for (const ch of RHYTHM_CHANNELS) {
    const steps = grid[ch].map(on => (on ? 'x' : '.')).join(' ');
    lines.push(`  ${ch.padEnd(3)} ${steps}`);
  }
  lines.push('');
  return lines.join('\n');
}

/** Replace (or insert) the @RHYTHM block in the full document content. */
function spliceRhythmSection(content: string, newBlock: string): string {
  // Remove existing @RHYTHM block
  const cleaned = content.replace(/@RHYTHM[\s\S]*?(?=\n@|\n#|\n\/\/|$)/g, '');
  return cleaned.trimEnd() + '\n\n' + newBlock;
}

// ──────────────────────────────────────────────────────────────────────────────

const PmdRhythmEditorPanel: React.FC = () => {
  const { activeDocumentId, documents, updateDocumentContent } = useDocumentStore(
    useShallow((state) => ({
      activeDocumentId: state.activeDocumentId,
      documents: state.documents,
      updateDocumentContent: state.updateDocumentContent,
    }))
  );

  const activeDocument = activeDocumentId ? documents.get(activeDocumentId) : null;

  // Initialise grid from the document on first render / document change
  const [grid, setGrid] = useState<Grid>(() =>
    activeDocument ? parseRhythmSection(activeDocument.content) : emptyGrid()
  );

  // Playhead step for visual feedback (reserved for future playback integration)
  const [playStep] = useState<number | null>(null);

  const toggleStep = useCallback((ch: RhythmChannel, step: number) => {
    setGrid(prev => {
      const next: Grid = {
        ...prev,
        [ch]: prev[ch].map((v, i) => (i === step ? !v : v)),
      };
      return next;
    });
  }, []);

  const handleApply = useCallback(() => {
    if (!activeDocumentId || !activeDocument) return;
    const block = serializeGrid(grid);
    const updated = spliceRhythmSection(activeDocument.content, block);
    updateDocumentContent(activeDocumentId, updated);
  }, [activeDocumentId, activeDocument, grid, updateDocumentContent]);

  const handleReset = useCallback(() => {
    setGrid(emptyGrid());
  }, []);

  const handleParse = useCallback(() => {
    if (!activeDocument) return;
    setGrid(parseRhythmSection(activeDocument.content));
  }, [activeDocument]);

  const cellStyle = (on: boolean, isPlay: boolean): React.CSSProperties => ({
    width: 18,
    height: 18,
    border: `1px solid ${on ? 'var(--accent, #569cd6)' : 'var(--border, #444)'}`,
    borderRadius: 2,
    backgroundColor: on
      ? 'var(--accent, #569cd6)'
      : isPlay
        ? 'var(--bg-highlight, #333)'
        : 'var(--bg-secondary, #252526)',
    cursor: 'pointer',
    display: 'inline-block',
    margin: 1,
    verticalAlign: 'middle',
    boxSizing: 'border-box',
  });

  return (
    <div style={{ padding: '6px', fontSize: '12px', overflowY: 'auto', height: '100%', boxSizing: 'border-box' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6 }}>
        <h3 style={{ margin: 0, fontSize: '13px' }}>PMD Rhythm Editor</h3>
        <button className="button small" onClick={handleParse} title="Load @RHYTHM from document">
          Parse
        </button>
        <button className="button small" onClick={handleApply} title="Write @RHYTHM back to document">
          Apply
        </button>
        <button className="button small secondary" onClick={handleReset} title="Clear all steps">
          Clear
        </button>
      </div>

      {/* Step ruler */}
      <div style={{ display: 'flex', alignItems: 'center', marginBottom: 2, marginLeft: 46 }}>
        {Array.from({ length: STEPS }, (_, i) => (
          <span
            key={i}
            style={{
              width: 20,
              textAlign: 'center',
              fontSize: 9,
              color: 'var(--text-muted, #666)',
              display: 'inline-block',
            }}
          >
            {i + 1}
          </span>
        ))}
      </div>

      {/* Grid */}
      {RHYTHM_CHANNELS.map(ch => (
        <div key={ch} style={{ display: 'flex', alignItems: 'center', marginBottom: 2 }}>
          <span
            style={{
              width: 44,
              fontSize: 10,
              fontWeight: 600,
              color: 'var(--text-muted, #888)',
              flexShrink: 0,
            }}
            title={CHANNEL_LABELS[ch]}
          >
            {ch}
          </span>
          {grid[ch].map((on, step) => (
            <span
              key={step}
              style={cellStyle(on, playStep === step)}
              onClick={() => toggleStep(ch, step)}
              title={`${ch} step ${step + 1}: ${on ? 'on' : 'off'}`}
              role="checkbox"
              aria-checked={on}
              tabIndex={0}
              onKeyDown={e => e.key === ' ' || e.key === 'Enter' ? toggleStep(ch, step) : undefined}
            />
          ))}
        </div>
      ))}

      <div style={{ marginTop: 8, color: 'var(--text-muted, #666)', fontSize: 11, lineHeight: 1.4 }}>
        Click a cell to toggle a step. Press <strong>Parse</strong> to load from
        the <code>@RHYTHM</code> section in the document, and <strong>Apply</strong> to
        write it back.
      </div>
    </div>
  );
};

export default PmdRhythmEditorPanel;
