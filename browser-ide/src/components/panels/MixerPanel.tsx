import React from 'react';
import type { SoundChip } from '@/types';

interface ChipVolume {
  chip: SoundChip;
  volume: number;
  pan: number;
  muted: boolean;
  solo: boolean;
}

const MixerPanel: React.FC = () => {
  // Mock mixer data - TODO: Connect to audio player state
  const [chips, setChips] = React.useState<ChipVolume[]>([
    { chip: 'YM2608', volume: 100, pan: 50, muted: false, solo: false },
    { chip: 'SN76489', volume: 100, pan: 50, muted: false, solo: false },
  ]);

  const handleVolumeChange = (index: number, value: number) => {
    const newChips = [...chips];
    newChips[index].volume = value;
    setChips(newChips);
    console.log(`Volume change for ${newChips[index].chip}: ${value}`);
  };

  const handlePanChange = (index: number, value: number) => {
    const newChips = [...chips];
    newChips[index].pan = value;
    setChips(newChips);
    console.log(`Pan change for ${newChips[index].chip}: ${value}`);
  };

  const handleToggleMute = (index: number) => {
    const newChips = [...chips];
    newChips[index].muted = !newChips[index].muted;
    setChips(newChips);
    console.log(`Toggle mute for ${newChips[index].chip}`);
  };

  const handleToggleSolo = (index: number) => {
    const newChips = [...chips];
    // Clear other solos if not ctrl-click (simplified for now)
    newChips.forEach((_, i) => {
      newChips[i].solo = (i === index) && !newChips[i].solo;
    });
    setChips(newChips);
    console.log(`Toggle solo for ${newChips[index].chip}`);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', padding: '4px' }}>
      <div style={{ fontWeight: 'bold', marginBottom: '4px', fontSize: '11px' }}>
        Mixer
      </div>
      
      <div style={{ flex: 1, overflowY: 'auto' }}>
        {chips.map((chipData, index) => (
          <div
            key={index}
            style={{
              display: 'flex',
              alignItems: 'center',
              padding: '4px',
              borderBottom: '1px solid var(--border-color)',
              gap: '4px',
            }}
          >
            {/* Chip name */}
            <span style={{ width: '60px', fontSize: '11px' }}>
              {chipData.chip}
            </span>

            {/* Volume slider */}
            <div style={{ flex: 1, display: 'flex', alignItems: 'center', gap: '4px' }}>
              <span style={{ fontSize: '10px', color: 'var(--text-muted)' }}>Vol</span>
              <input
                type="range"
                min="0"
                max="127"
                value={chipData.volume}
                onChange={(e) => handleVolumeChange(index, parseInt(e.target.value))}
                style={{ flex: 1, height: '4px' }}
              />
              <span style={{ width: '30px', fontSize: '10px', textAlign: 'right' }}>
                {chipData.volume}
              </span>
            </div>

            {/* Pan slider */}
            <div style={{ flex: 1, display: 'flex', alignItems: 'center', gap: '4px' }}>
              <span style={{ fontSize: '10px', color: 'var(--text-muted)' }}>Pan</span>
              <input
                type="range"
                min="0"
                max="127"
                value={chipData.pan}
                onChange={(e) => handlePanChange(index, parseInt(e.target.value))}
                style={{ flex: 1, height: '4px' }}
              />
              <span style={{ width: '30px', fontSize: '10px', textAlign: 'right' }}>
                {chipData.pan}
              </span>
            </div>

            {/* Mute/Solo buttons */}
            <div style={{ display: 'flex', gap: '2px' }}>
              <button
                onClick={() => handleToggleMute(index)}
                style={{
                  fontSize: '10px',
                  padding: '2px 4px',
                  background: chipData.muted ? 'var(--button-active-bg)' : 'var(--button-bg)',
                  color: chipData.muted ? 'var(--button-active-fg)' : 'var(--button-fg)',
                  border: '1px solid var(--border-color)',
                  cursor: 'pointer',
                }}
              >
                M
              </button>
              <button
                onClick={() => handleToggleSolo(index)}
                style={{
                  fontSize: '10px',
                  padding: '2px 4px',
                  background: chipData.solo ? 'var(--button-active-bg)' : 'var(--button-bg)',
                  color: chipData.solo ? 'var(--button-active-fg)' : 'var(--button-fg)',
                  border: '1px solid var(--border-color)',
                  cursor: 'pointer',
                }}
              >
                S
              </button>
            </div>
          </div>
        ))}
      </div>

      {/* Master volume */}
      <div style={{ padding: '4px', borderTop: '1px solid var(--border-color)', marginTop: 'auto' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
          <span style={{ width: '60px', fontSize: '11px' }}>Master</span>
          <input
            type="range"
            min="0"
            max="127"
            value={100}
            onChange={(e) => console.log(`Master volume: ${e.target.value}`)}
            style={{ flex: 1, height: '4px' }}
          />
          <span style={{ width: '30px', fontSize: '10px', textAlign: 'right' }}>100</span>
        </div>
      </div>
    </div>
  );
};

export default MixerPanel;
