import React, { useState, useCallback, useEffect } from 'react';
import type { OutputFormat, SoundChip } from '@/types';
import { useCompileStore } from '@/stores/compileStore';
import { useDocumentStore } from '@/stores/documentStore';

const CompileOptionsPanel: React.FC = () => {
  const [selectedFormat, setSelectedFormat] = useState<OutputFormat>('vgm');
  const [selectedChips, setSelectedChips] = useState<SoundChip[]>(['YM2608', 'SN76489']);
  const [clockCount, setClockCount] = useState(0);
  const [compression, setCompression] = useState(0);
  const [optimize, setOptimize] = useState(true);
  const [verbose, setVerbose] = useState(false);
  const [debug, setDebug] = useState(false);
  const [compileStartedAt, setCompileStartedAt] = useState<number | null>(null);
  const [elapsedMs, setElapsedMs] = useState(0);

  // Get compile function and active document from stores
  const { compile, status, progress, progressMessage, lastCompileTimingSummary } = useCompileStore((state) => ({
    compile: state.compile,
    status: state.status,
    progress: state.progress,
    progressMessage: state.progressMessage,
    lastCompileTimingSummary: state.lastCompileTimingSummary,
  }));
    useEffect(() => {
      if (status === 'compiling') {
        if (!compileStartedAt) {
          setCompileStartedAt(Date.now());
        }

        const interval = setInterval(() => {
          setElapsedMs(compileStartedAt ? Date.now() - compileStartedAt : 0);
        }, 200);

        return () => clearInterval(interval);
      }

      setCompileStartedAt(null);
      setElapsedMs(0);
    }, [status, compileStartedAt]);

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

  const clockCountPresets = [
    { value: 0, label: 'Auto (driver/MML default)' },
    { value: 96, label: '96 (half resolution)' },
    { value: 192, label: '192 (common default)' },
    { value: 384, label: '384 (higher timing resolution)' },
    { value: 768, label: '768 (very high timing resolution)' },
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

  const toBrowserTargetChips = (chips: SoundChip[]): string[] => {
    const normalized = chips.map((chip) => chip.toLowerCase());
    if (normalized.length === 1 && normalized[0] === 'ym2608') {
      return ['ym2608', 'sn76489'];
    }
    return normalized;
  };

  // Handle compile
  const handleCompile = useCallback(async () => {
    if (!activeDocumentId || status === 'compiling') return;

    try {
      const options: any = {
        format: selectedFormat,
        target_chips: toBrowserTargetChips(selectedChips),
        clock_count: clockCount,
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
  }, [activeDocumentId, compile, status, selectedFormat, selectedChips, clockCount, compression, optimize, verbose, debug]);

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

      {/* Clock Count */}
      <div style={{ marginBottom: '8px' }}>
        <label style={{ display: 'block', fontSize: '12px', marginBottom: '4px' }}>
          Clock Count Override
        </label>
        <select
          value={clockCount}
          onChange={(e) => setClockCount(Number(e.target.value))}
          style={{ width: '100%', padding: '4px', fontSize: '12px' }}
        >
          {clockCountPresets.map((preset) => (
            <option key={preset.value} value={preset.value}>
              {preset.label}
            </option>
          ))}
        </select>
        <div style={{ marginTop: '4px', fontSize: '11px', color: 'var(--text-muted)' }}>
          Use Auto unless you need explicit MML clock-count behavior tuning.
        </div>
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

      {/* Compile Diagnostics */}
      <div style={{ marginTop: '8px', backgroundColor: 'var(--bg-tertiary)', padding: '6px', borderRadius: '3px', border: '1px solid var(--border-color)' }}>
        <div style={{ fontSize: '12px', marginBottom: '4px' }}><strong>Compile Diagnostics</strong></div>
        <div style={{ fontSize: '11px', marginBottom: '2px' }}>
          <span style={{ color: 'var(--text-muted)' }}>Status:</span> <span>{status}</span>
          {status === 'compiling' && typeof progress === 'number' && (
            <span>{` (${Math.round(progress)}%)`}</span>
          )}
        </div>
        {status === 'compiling' && (
          <div style={{ fontSize: '11px', marginBottom: '2px' }}>
            <span style={{ color: 'var(--text-muted)' }}>Elapsed:</span> <span>{(elapsedMs / 1000).toFixed(1)}s</span>
          </div>
        )}
        {progressMessage && (
          <div style={{ fontSize: '11px', marginBottom: '2px', whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
            <span style={{ color: 'var(--text-muted)' }}>Phase:</span> <span>{progressMessage}</span>
          </div>
        )}
        {lastCompileTimingSummary && status !== 'compiling' && (
          <div style={{ fontSize: '11px', whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
            <span style={{ color: 'var(--text-muted)' }}>Last:</span> <span>{lastCompileTimingSummary}</span>
          </div>
        )}
      </div>
    </div>
  );
};

export default CompileOptionsPanel;
