/**
 * MoonDriver Chip Selector Panel
 *
 * Lets the user choose which FM synthesis chip variant a MoonDriver (.mdl)
 * document targets: OPN2 (YM2612), OPNA (YM2608), or OPN3 (YMF288).
 *
 * The selection rewrites the chip directive in the document on Apply.
 */

import React, { useState, useCallback } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { useDocumentStore } from '@/stores/documentStore';

// ─── Chip definitions ─────────────────────────────────────────────────────────

type ChipVariant = 'OPN2' | 'OPNA' | 'OPN3';

interface ChipInfo {
  variant: ChipVariant;
  part: string;
  description: string;
  channels: number;
  extraFeatures: string[];
}

const CHIPS: ChipInfo[] = [
  {
    variant: 'OPN2',
    part: 'YM2612',
    description: 'Sega Mega Drive / Genesis FM chip',
    channels: 6,
    extraFeatures: ['DAC channel (ch6)', 'Stereo LFO'],
  },
  {
    variant: 'OPNA',
    part: 'YM2608',
    description: 'NEC PC-98 FM chip with ADPCM / rhythm',
    channels: 6,
    extraFeatures: ['ADPCM-B', 'Rhythm (6 ch)', 'SSG (3 ch)'],
  },
  {
    variant: 'OPN3',
    part: 'YMF288',
    description: 'PC-98 OPN3 (OPN2 + OPNA rhythm)',
    channels: 6,
    extraFeatures: ['ADPCM-B', 'Rhythm (6 ch)'],
  },
];

// ─── Parse / serialise ────────────────────────────────────────────────────────

const CHIP_DIRECTIVES: ChipVariant[] = ['OPN2', 'OPNA', 'OPN3'];

function detectChip(content: string): ChipVariant {
  for (const v of CHIP_DIRECTIVES) {
    if (new RegExp(`#${v}\\b`, 'i').test(content)) return v;
  }
  return 'OPN2'; // default
}

/** Replace or insert the chip directive in document content. */
function applyChipDirective(content: string, chip: ChipVariant): string {
  // Remove all existing chip directives
  let updated = content;
  for (const v of CHIP_DIRECTIVES) {
    updated = updated.replace(new RegExp(`^#${v}\\b.*$`, 'gim'), '');
  }
  // Remove blank lines left at the top
  updated = updated.replace(/^\n+/, '');
  // Prepend new directive
  return `#${chip}\n${updated}`;
}

// ─── Panel ─────────────────────────────────────────────────────────────────────

const MoonDriverChipSelectorPanel: React.FC = () => {
  const { activeDocumentId, documents, updateDocumentContent } = useDocumentStore(
    useShallow((state) => ({
      activeDocumentId: state.activeDocumentId,
      documents: state.documents,
      updateDocumentContent: state.updateDocumentContent,
    }))
  );

  const activeDocument = activeDocumentId ? documents.get(activeDocumentId) : null;

  const [selected, setSelected] = useState<ChipVariant>(() =>
    activeDocument ? detectChip(activeDocument.content) : 'OPN2'
  );

  const handleApply = useCallback(() => {
    if (!activeDocumentId || !activeDocument) return;
    const updated = applyChipDirective(activeDocument.content, selected);
    updateDocumentContent(activeDocumentId, updated);
  }, [activeDocumentId, activeDocument, selected, updateDocumentContent]);

  const selectedChip = CHIPS.find(c => c.variant === selected)!;

  return (
    <div style={{ padding: '6px', fontSize: '12px', overflowY: 'auto', height: '100%', boxSizing: 'border-box' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
        <h3 style={{ margin: 0, fontSize: '13px' }}>MoonDriver Chip</h3>
        <button className="button small" onClick={handleApply}>Apply</button>
      </div>

      {/* Radio buttons */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
        {CHIPS.map(chip => (
          <label
            key={chip.variant}
            style={{
              display: 'flex',
              alignItems: 'flex-start',
              gap: 8,
              cursor: 'pointer',
              padding: '6px 8px',
              borderRadius: 4,
              border: `1px solid ${selected === chip.variant ? 'var(--accent, #569cd6)' : 'var(--border, #444)'}`,
              background: selected === chip.variant ? 'var(--bg-selected, #1a3a5c)' : 'var(--bg-secondary, #252526)',
            }}
          >
            <input
              type="radio"
              name="moondriver-chip"
              value={chip.variant}
              checked={selected === chip.variant}
              onChange={() => setSelected(chip.variant)}
              style={{ marginTop: 2 }}
            />
            <div>
              <div style={{ fontWeight: 600, color: 'var(--text, #ccc)' }}>
                #{chip.variant}
                <span style={{ marginLeft: 8, fontWeight: 400, color: 'var(--text-muted, #888)' }}>
                  ({chip.part})
                </span>
              </div>
              <div style={{ color: 'var(--text-muted, #888)', marginTop: 2 }}>{chip.description}</div>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4, marginTop: 4 }}>
                <span style={{ fontSize: 10, background: 'var(--bg-tertiary, #333)', borderRadius: 3, padding: '1px 5px', color: 'var(--text-muted, #888)' }}>
                  {chip.channels} FM channels
                </span>
                {chip.extraFeatures.map(f => (
                  <span key={f} style={{ fontSize: 10, background: 'var(--bg-tertiary, #333)', borderRadius: 3, padding: '1px 5px', color: 'var(--text-muted, #888)' }}>
                    {f}
                  </span>
                ))}
              </div>
            </div>
          </label>
        ))}
      </div>

      {/* Preview of generated directive */}
      <div style={{ marginTop: 10 }}>
        <div style={{ fontSize: 11, color: 'var(--text-muted, #888)', marginBottom: 3 }}>
          Directive that will be written to document:
        </div>
        <code style={{ fontSize: 12, color: 'var(--accent, #569cd6)' }}>#{selectedChip.variant}</code>
      </div>

      <div style={{ marginTop: 8, color: 'var(--text-muted, #666)', fontSize: 11, lineHeight: 1.4 }}>
        Select the target FM chip and press <strong>Apply</strong> to update the
        <code> #OPN2</code> / <code>#OPNA</code> / <code>#OPN3</code> directive at
        the top of your MoonDriver file.
      </div>
    </div>
  );
};

export default MoonDriverChipSelectorPanel;
