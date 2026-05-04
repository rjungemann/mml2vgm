import React from 'react';
import type { PartInfo, SoundChip } from '@/types';

const PartCounterPanel: React.FC = () => {
  // Mock part data for now
  // TODO: Connect to actual compile results to extract part info
  const parts: PartInfo[] = [
    { index: 0, name: 'FM1', chip: 'YM2608' as SoundChip, channel: 0, volume: 100, pan: 0, isSolo: false, isMuted: false, isKbdAssigned: false },
    { index: 1, name: 'FM2', chip: 'YM2608' as SoundChip, channel: 1, volume: 100, pan: 0, isSolo: false, isMuted: false, isKbdAssigned: false },
    { index: 2, name: 'FM3', chip: 'YM2608' as SoundChip, channel: 2, volume: 100, pan: 0, isSolo: false, isMuted: false, isKbdAssigned: false },
    { index: 3, name: 'SSG1', chip: 'AY8910' as SoundChip, channel: 0, volume: 80, pan: 0, isSolo: false, isMuted: false, isKbdAssigned: false },
    { index: 4, name: 'SSG2', chip: 'AY8910' as SoundChip, channel: 1, volume: 80, pan: 0, isSolo: false, isMuted: false, isKbdAssigned: false },
    { index: 5, name: 'SSG3', chip: 'AY8910' as SoundChip, channel: 2, volume: 80, pan: 0, isSolo: false, isMuted: false, isKbdAssigned: false },
  ];

  // Get stats
  const totalParts = parts.length;
  const activeParts = parts.filter(p => !p.isMuted).length;
  const mutedParts = parts.filter(p => p.isMuted).length;
  const soloParts = parts.filter(p => p.isSolo).length;

  // Toggle mute for a part
  const handleToggleMute = (index: number) => {
    console.log(`Toggle mute for part ${index}`);
  };

  // Toggle solo for a part
  const handleToggleSolo = (index: number) => {
    console.log(`Toggle solo for part ${index}`);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      {/* Summary */}
      <div style={{ padding: '4px 8px', borderBottom: '1px solid var(--border-color)' }}>
        <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
          Total: {totalParts}
        </div>
        <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
          Active: {activeParts}
        </div>
        <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
          Muted: {mutedParts}
        </div>
        <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
          Solo: {soloParts}
        </div>
      </div>

      {/* Part list */}
      <div style={{ flex: 1, overflowY: 'auto', padding: '4px' }}>
        {parts.map((part) => (
          <div
            key={part.index}
            style={{
              display: 'flex',
              alignItems: 'center',
              height: '24px',
              padding: '0 4px',
              borderRadius: '2px',
              cursor: 'pointer',
              backgroundColor: part.isMuted ? 'var(--bg-tertiary)' : 'transparent',
            }}
            onClick={() => handleToggleMute(part.index)}
          >
            <span style={{ width: '20px', textAlign: 'center', fontSize: '11px' }}>
              {part.index}
            </span>
            <span style={{ flex: 1, margin: '0 4px', fontSize: '12px' }}>
              {part.name}
            </span>
            <span style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
              {part.chip}
            </span>
            <span style={{ marginLeft: '4px' }}>
              <button
                style={{
                  width: '18px',
                  height: '18px',
                  border: 'none',
                  backgroundColor: part.isMuted ? 'var(--accent-warning)' : 'transparent',
                  color: part.isMuted ? 'black' : 'var(--text-muted)',
                  cursor: 'pointer',
                  fontSize: '10px',
                }}
                onClick={(e) => {
                  e.stopPropagation();
                  handleToggleMute(part.index);
                }}
              >
                M
              </button>
              <button
                style={{
                  width: '18px',
                  height: '18px',
                  border: 'none',
                  backgroundColor: part.isSolo ? 'var(--accent-primary)' : 'transparent',
                  color: part.isSolo ? 'white' : 'var(--text-muted)',
                  cursor: 'pointer',
                  fontSize: '10px',
                }}
                onClick={(e) => {
                  e.stopPropagation();
                  handleToggleSolo(part.index);
                }}
              >
                S
              </button>
            </span>
          </div>
        ))}
      </div>

      {/* Controls */}
      <div style={{ padding: '4px 8px', borderTop: '1px solid var(--border-color)' }}>
        <button
          className="button small"
          onClick={() => parts.forEach(() => console.log(`Mute all`))}
        >
          Mute All
        </button>
        <button
          className="button small"
          onClick={() => parts.forEach(() => console.log(`Unmute all`))}
        >
          Unmute All
        </button>
        <button
          className="button small"
          onClick={() => parts.forEach(() => console.log(`Solo all`))}
        >
          Clear Solo
        </button>
      </div>
    </div>
  );
};

export default PartCounterPanel;
