import React, { useEffect, useState, useCallback } from 'react';
import type { SoundChip } from '@/types';
import { audioService } from '@/services/audioService';

interface ChipVolumeState {
  chip: SoundChip;
  volume: number; // 0-127 for UI, converted to 0-1 for audio
  pan: number; // 0-127
  muted: boolean;
  solo: boolean;
}

const MixerPanel: React.FC = () => {
  // Get all supported chips for the mixer
  const allChips: SoundChip[] = [
    'YM2608', 'SN76489', 'YM2612', 'YM2151', 'YM2203', 'YM3526', 'YM3812', 'YMF262',
    'YM2413', 'RF5C164', 'SegaPCM', 'AY8910', 'HuC6280'
  ];
  
  // Initialize mixer state from audioService
  const [chips, setChips] = useState<ChipVolumeState[]>(() => {
    return allChips.map(chip => ({
      chip,
      volume: 100, // Default to 100 (full volume)
      pan: 50, // Default to center
      muted: false,
      solo: false,
    }));
  });

  // Subscribe to audio service changes
  useEffect(() => {
    // Update chip list based on currently active chips
    const status = audioService.getStatus();
    if (status.chips.length > 0) {
      // Add any new chips that aren't in our list
      setChips(prev => {
        const existingChipNames = new Set(prev.map(c => c.chip));
        const newChips: ChipVolumeState[] = [];
        
        // Keep existing chips
        for (const chipState of prev) {
          newChips.push(chipState);
        }
        
        // Add new chips from audio service
        for (const chip of status.chips) {
          if (!existingChipNames.has(chip)) {
            newChips.push({
              chip,
              volume: 100,
              pan: 50,
              muted: false,
              solo: false,
            });
          }
        }
        
        return newChips;
      });
    }
  }, []);

  // Handle volume change
  const handleVolumeChange = useCallback((index: number, value: number) => {
    setChips(prev => {
      const newChips = [...prev];
      newChips[index].volume = value;
      
      // Update audioService
      const chip = newChips[index].chip;
      audioService.setChipVolume(chip, value / 127);
      
      return newChips;
    });
  }, []);

  // Handle pan change
  const handlePanChange = useCallback((index: number, value: number) => {
    setChips(prev => {
      const newChips = [...prev];
      newChips[index].pan = value;
      // TODO: Implement pan in audioService
      console.log(`Pan change for ${newChips[index].chip}: ${value}`);
      return newChips;
    });
  }, []);

  // Handle toggle mute
  const handleToggleMute = useCallback((index: number) => {
    setChips(prev => {
      const newChips = [...prev];
      newChips[index].muted = !newChips[index].muted;
      
      // Update audioService
      const chip = newChips[index].chip;
      audioService.setChipMuted(chip, newChips[index].muted);
      
      return newChips;
    });
  }, []);

  // Handle toggle solo
  const handleToggleSolo = useCallback((index: number, ctrlKey: boolean = false) => {
    setChips(prev => {
      const newChips = [...prev];
      
      if (!ctrlKey) {
        // Clear all solos first
        newChips.forEach(c => c.solo = false);
      }
      
      // Toggle the clicked chip
      newChips[index].solo = !newChips[index].solo;
      
      // Update audioService
      const chip = newChips[index].chip;
      audioService.setChipSolo(chip, newChips[index].solo);
      
      return newChips;
    });
  }, []);

  // Handle master volume change
  const handleMasterVolumeChange = useCallback((value: number) => {
    audioService.setVolume(value / 127);
  }, []);

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
                onClick={(e) => handleToggleSolo(index, e.ctrlKey || e.metaKey)}
                style={{
                  fontSize: '10px',
                  padding: '2px 4px',
                  background: chipData.solo ? 'var(--button-active-bg)' : 'var(--button-bg)',
                  color: chipData.solo ? 'var(--button-active-fg)' : 'var(--button-fg)',
                  border: '1px solid var(--border-color)',
                  cursor: 'pointer',
                }}
                title="Hold Ctrl to toggle multiple solos"
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
            value={Math.round(audioService.getVolume() * 127)}
            onChange={(e) => handleMasterVolumeChange(parseInt(e.target.value))}
            style={{ flex: 1, height: '4px' }}
          />
          <span style={{ width: '30px', fontSize: '10px', textAlign: 'right' }}>
            {Math.round(audioService.getVolume() * 127)}
          </span>
        </div>
      </div>
    </div>
  );
};

export default MixerPanel;
