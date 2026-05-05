import React, { useState, useCallback } from 'react';
import type { OutputFormat, SoundChip } from '@/types';
import { useCompileStore } from '@/stores/compileStore';
import { useDocumentStore } from '@/stores/documentStore';

const CompileOptionsPanel: React.FC = () => {
  const [selectedFormat, setSelectedFormat] = useState<OutputFormat>('vgm');
  const [selectedChips, setSelectedChips] = useState<SoundChip[]>(['YM2608']);
  const [clockRate, setClockRate] = useState(7987200);
  const [compression, setCompression] = useState(0);
  const [optimize, setOptimize] = useState(true);
  const [verbose, setVerbose] = useState(false);
  const [debug, setDebug] = useState(false);

  // Get compile function and active document from stores
  const { compile, status } = useCompileStore((state) => ({
    compile: state.compile,
    status: state.status,
  }));
  const { activeDocumentId } = useDocumentStore((state) => ({
    activeDocumentId: state.activeDocumentId,
  }));

  // Available formats
  const formats: { value: OutputFormat; label: string }[] = [
    { value: 'vgm', label: 'VGM' },
    { value: 'xgm', label: 'XGM' },
    { value: 'xgm2', label: 'XGM2' },
    { value: 'zgm', label: 'ZGM' },
  ];

  // Available chips
  const chips: { value: SoundChip; label: string; group?: string }[] = [
    { value: 'YM2608', label: 'YM2608 (OPNA)', group: 'FM' },
    { value: 'YM2609', label: 'YM2609 (OPNB)', group: 'FM' },
    { value: 'YM2612', label: 'YM2612 (OPN2)', group: 'FM' },
    { value: 'YM2612X', label: 'YM2612 Extended', group: 'FM' },
    { value: 'YM2610B', label: 'YM2610B (OPNB)', group: 'FM' },
    { value: 'YM2151', label: 'YM2151 (OPM)', group: 'FM' },
    { value: 'YM3526', label: 'YM3526 (OPL)', group: 'FM' },
    { value: 'Y8950', label: 'Y8950 (OPL)', group: 'FM' },
    { value: 'YM3812', label: 'YM3812 (OPL2)', group: 'FM' },
    { value: 'YMF262', label: 'YMF262 (OPL3)', group: 'FM' },
    { value: 'YM2413', label: 'YM2413 (OPLL)', group: 'FM' },
    { value: 'YM2203', label: 'YM2203 (OPN)', group: 'FM' },
    { value: 'SN76489', label: 'SN76489 (PSG)', group: 'PSG' },
    { value: 'SN76489X2', label: 'SN76489 x2', group: 'PSG' },
    { value: 'AY8910', label: 'AY8910 (SSG)', group: 'PSG' },
    { value: 'RF5C164', label: 'RF5C164 (PCM)', group: 'PCM' },
    { value: 'SegaPCM', label: 'Sega PCM', group: 'PCM' },
    { value: 'HuC6280', label: 'HuC6280', group: 'Other' },
    { value: 'C140', label: 'C140', group: 'Other' },
    { value: 'C352', label: 'C352', group: 'Other' },
    { value: 'K051649', label: 'K051649', group: 'Konami' },
    { value: 'K053260', label: 'K053260', group: 'Konami' },
    { value: 'K054539', label: 'K054539', group: 'Konami' },
    { value: 'QSound', label: 'QSound', group: 'Other' },
    { value: 'NES', label: 'NES APU', group: 'Console' },
    { value: 'DMG', label: 'Game Boy APU', group: 'Console' },
    { value: 'VRC6', label: 'VRC6', group: 'Console' },
    { value: 'POKEY', label: 'POKEY', group: 'Console' },
    { value: 'MIDI', label: 'MIDI Output', group: 'Other' },
  ];

  // Toggle chip selection
  const toggleChip = (chip: SoundChip) => {
    setSelectedChips((prev) => {
      if (prev.includes(chip)) {
        return prev.filter((c) => c !== chip);
      }
      return [...prev, chip];
    });
  };

  // Handle compile
  const handleCompile = useCallback(async () => {
    if (!activeDocumentId || status === 'compiling') return;

    try {
      const options: any = {
        format: selectedFormat,
        target_chips: selectedChips,
        clock_count: clockRate,
        compression,
        optimize,
        verbose,
        debug,
      };

      console.log('Compiling with options:', options);
      await compile(activeDocumentId, options);
    } catch (error) {
      console.error('Compilation error:', error);
    }
  }, [activeDocumentId, compile, status, selectedFormat, selectedChips, clockRate, compression, optimize, verbose, debug]);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', padding: '4px' }}>
      {/* Output Format */}
      <div style={{ marginBottom: '8px' }}>
        <label style={{ display: 'block', fontSize: '12px', marginBottom: '4px' }}>
          Output Format
        </label>
        <select
          value={selectedFormat}
          onChange={(e) => setSelectedFormat(e.target.value as OutputFormat)}
          style={{ width: '100%', padding: '4px', fontSize: '12px' }}
        >
          {formats.map((f) => (
            <option key={f.value} value={f.value}>
              {f.label}
            </option>
          ))}
        </select>
      </div>

      {/* Sound Chips */}
      <div style={{ marginBottom: '8px' }}>
        <label style={{ display: 'block', fontSize: '12px', marginBottom: '4px' }}>
          Sound Chips
        </label>
        <div 
          style={{
            maxHeight: '150px',
            overflowY: 'auto',
            border: '1px solid var(--border-color)',
            borderRadius: '3px',
          }}
        >
          {Object.entries(
            chips.reduce((acc, chip) => {
              if (!chip.group) chip.group = 'Ungrouped';
              if (!acc[chip.group]) acc[chip.group] = [];
              acc[chip.group].push(chip);
              return acc;
            }, {} as Record<string, typeof chips>)
          ).map(([group, groupChips]) => (
            <div key={group} style={{ padding: '4px' }}>
              <div style={{ fontSize: '11px', color: 'var(--text-muted)', marginBottom: '2px' }}>
                {group}
              </div>
              {groupChips.map((chip) => (
                <div
                  key={chip.value}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    padding: '2px 4px',
                    cursor: 'pointer',
                    backgroundColor: selectedChips.includes(chip.value)
                      ? 'var(--highlight-line)'
                      : 'transparent',
                    borderRadius: '2px',
                  }}
                  onClick={() => toggleChip(chip.value)}
                >
                  <input
                    type="checkbox"
                    checked={selectedChips.includes(chip.value)}
                    onChange={() => toggleChip(chip.value)}
                    onClick={(e) => e.stopPropagation()}
                    style={{ marginRight: '4px' }}
                  />
                  <span style={{ fontSize: '11px' }}>{chip.label}</span>
                </div>
              ))}
            </div>
          ))}
        </div>
      </div>

      {/* Clock Rate */}
      <div style={{ marginBottom: '8px' }}>
        <label style={{ display: 'block', fontSize: '12px', marginBottom: '4px' }}>
          Clock Rate (Hz)
        </label>
        <input
          type="number"
          value={clockRate}
          onChange={(e) => setClockRate(Number(e.target.value))}
          style={{ width: '100%', padding: '4px', fontSize: '12px' }}
        />
      </div>

      {/* Options */}
      <div style={{ marginBottom: '8px' }}>
        <label style={{ display: 'block', fontSize: '12px', marginBottom: '4px' }}>
          Options
        </label>
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '4px' }}>
          <label style={{ display: 'flex', alignItems: 'center', fontSize: '11px' }}>
            <input
              type="checkbox"
              checked={optimize}
              onChange={(e) => setOptimize(e.target.checked)}
              style={{ marginRight: '4px' }}
            />
            Optimize
          </label>
          <label style={{ display: 'flex', alignItems: 'center', fontSize: '11px' }}>
            <input
              type="checkbox"
              checked={verbose}
              onChange={(e) => setVerbose(e.target.checked)}
              style={{ marginRight: '4px' }}
            />
            Verbose
          </label>
          <label style={{ display: 'flex', alignItems: 'center', fontSize: '11px' }}>
            <input
              type="checkbox"
              checked={debug}
              onChange={(e) => setDebug(e.target.checked)}
              style={{ marginRight: '4px' }}
            />
            Debug
          </label>
        </div>
      </div>

      {/* Compression */}
      <div style={{ marginBottom: '8px' }}>
        <label style={{ display: 'block', fontSize: '12px', marginBottom: '4px' }}>
          Compression Level: {compression}
        </label>
        <input
          type="range"
          min={0}
          max={9}
          value={compression}
          onChange={(e) => setCompression(Number(e.target.value))}
          style={{ width: '100%' }}
        />
      </div>

      {/* Actions */}
      <div style={{ display: 'flex', gap: '4px' }}>
        <button 
          className="button primary"
          onClick={handleCompile}
          style={{ flex: 1 }}
        >
          Compile
        </button>
      </div>
    </div>
  );
};

export default CompileOptionsPanel;
