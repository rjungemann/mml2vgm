import React, { useState, useEffect } from 'react';
import { midiService } from '@/services/midiService';
import { partService } from '@/services/partService';
import type { MidiNoteEvent } from '@/services/midiService';

interface MIDIKey {
  note: string;
  octave: number;
  midiNote: number;
  isBlack: boolean;
  active: boolean;
}

// Define white keys (C, D, E, F, G, A, B)
const WHITE_KEYS = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];

// Define black keys and their positions
const BLACK_KEYS: Record<string, number> = {
  'C#': 0.5,
  'D#': 1.5,
  'F#': 3.5,
  'G#': 4.5,
  'A#': 5.5,
};

const MIDIKeyboardPanel: React.FC = () => {
  // Get state from midiService
  const [midiState, setMidiState] = useState(midiService.getState());
  const [activeKeys, setActiveKeys] = React.useState<Set<number>>(new Set());
  const [octaveOffset, setOctaveOffset] = React.useState(3);
  
  // Get parts from partService for part selection
  const [parts, setParts] = useState(() => partService.getParts());
  
  // Derive mode and assigned part from midiService state
  const mode = midiState.mode;
  const assignedPartIndex = midiState.assignedPart;
  const isSupported = midiState.isSupported;
  const inputDevices = midiState.inputDevices;
  
  // Get assigned part name
  const getAssignedPartName = (): string => {
    if (assignedPartIndex === null) return 'None';
    const part = parts.find(p => p.index === assignedPartIndex);
    return part ? part.name : `Part ${assignedPartIndex + 1}`;
  };
  
  // Listen to midiService state changes
  useEffect(() => {
    const handleStateUpdate = (state: any) => {
      setMidiState(state);
    };
    
    midiService.addStateListener(handleStateUpdate);
    
    return () => {
      midiService.removeStateListener(handleStateUpdate);
    };
  }, []);
  
  // Listen to partService changes
  useEffect(() => {
    const handlePartsUpdate = (updatedParts: any[]) => {
      setParts(updatedParts);
    };
    
    partService.addListener(handlePartsUpdate);
    
    return () => {
      partService.removeListener(handlePartsUpdate);
    };
  }, []);
  
  // Listen to MIDI note events for key highlighting
  useEffect(() => {
    const handleNoteEvent = (event: MidiNoteEvent) => {
      if (event.type === 'noteOn' && event.velocity > 0) {
        // Highlight the key
        setActiveKeys(prev => new Set([...prev, event.note]));
      } else if (event.type === 'noteOff' || (event.type === 'noteOn' && event.velocity === 0)) {
        // Remove highlight
        setActiveKeys(prev => {
          const next = new Set(prev);
          next.delete(event.note);
          return next;
        });
      }
    };
    
    midiService.addListener(handleNoteEvent);
    
    return () => {
      midiService.removeListener(handleNoteEvent);
    };
  }, []);
  
  // Initialize MIDI on mount if supported
  useEffect(() => {
    if (isSupported && !midiState.isEnabled) {
      midiService.init().catch(console.error);
    }
  }, [isSupported, midiState.isEnabled]);
  
  // Cycle through parts for assignment
  const cycleAssignedPart = () => {
    if (parts.length === 0) {
      midiService.setAssignedPart(null);
      return;
    }
    
    const currentIndex = assignedPartIndex ?? -1;
    const nextIndex = (currentIndex + 1) % parts.length;
    midiService.setAssignedPart(parts[nextIndex].index);
  };
  
  // Generate keys for 2 octaves
  const generateKeys = (): MIDIKey[] => {
    const keys: MIDIKey[] = [];
    
    // Generate 2 octaves (from octaveOffset to octaveOffset + 1)
    for (let oct = octaveOffset; oct <= octaveOffset + 1; oct++) {
      // Add white keys
      WHITE_KEYS.forEach(note => {
        keys.push({
          note,
          octave: oct,
          midiNote: (oct + 1) * 12 + WHITE_KEYS.indexOf(note),
          isBlack: false,
          active: false,
        });
      });
      
      // Add black keys
      Object.entries(BLACK_KEYS).forEach(([note, pos]) => {
        keys.push({
          note,
          octave: oct,
          midiNote: (oct + 1) * 12 + Math.floor(pos),
          isBlack: true,
          active: false,
        });
      });
    }
    
    // Sort by MIDI note number
    return keys.sort((a, b) => a.midiNote - b.midiNote);
  };

  const keys = generateKeys();

  const handleKeyClick = (midiNote: number) => {
    // Highlight the key
    setActiveKeys(prev => new Set([...prev, midiNote]));
    
    // Send to midiService for handling
    if (isSupported) {
      // For now, just simulate the event through the service
      // In a real implementation, this would send MIDI messages
      // to the WASM chip player for preview
      console.log('[MIDI Keyboard] Note clicked:', midiNote, 'mode:', mode, 'part:', assignedPartIndex);
    }
  };

  // Key color based on state
  const getKeyColor = (key: MIDIKey) => {
    if (activeKeys.has(key.midiNote)) {
      return key.isBlack ? '#6aa84f' : '#388e3c';
    }
    return key.isBlack ? '#2d2d2d' : '#ffffff';
  };

  // Key text color
  const getKeyTextColor = (key: MIDIKey) => {
    return key.isBlack ? '#ffffff' : '#000000';
  };

  // Calculate key width and positioning
  const whiteKeyWidth = 30;
  const blackKeyWidth = 20;
  const blackKeyHeight = 120;
  const whiteKeyHeight = 200;

  // Render the keyboard
  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', padding: '4px' }}>
      {/* Header */}
      <div style={{ 
        display: 'flex', 
        justifyContent: 'space-between', 
        alignItems: 'center',
        padding: '4px 8px',
        borderBottom: '1px solid var(--border-color)',
        fontSize: '11px',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <span style={{ fontWeight: 'bold' }}>MIDI Keyboard</span>
          {!isSupported && (
            <span style={{ color: 'var(--text-warning)' }}>
              (Web MIDI API not supported)
            </span>
          )}
          {isSupported && !midiState.isEnabled && (
            <span style={{ color: 'var(--text-muted)' }}>
              (Not connected)
            </span>
          )}
          {isSupported && midiState.isEnabled && inputDevices.length > 0 && (
            <span style={{ color: 'var(--accent-primary)' }}>
              ✓ {inputDevices.length} device(s)
            </span>
          )}
        </div>
        <div style={{ display: 'flex', gap: '4px', alignItems: 'center' }}>
          <select
            value={mode}
            onChange={(e) => midiService.setMode(e.target.value as 'preview' | 'input')}
            style={{
              fontSize: '10px',
              padding: '2px',
              background: 'var(--input-bg)',
              color: 'var(--input-fg)',
              border: '1px solid var(--border-color)',
            }}
            disabled={!isSupported}
          >
            <option value="preview">Preview</option>
            <option value="input">Input</option>
          </select>
          <button
            onClick={cycleAssignedPart}
            style={{
              fontSize: '10px',
              padding: '2px 6px',
              background: 'var(--button-bg)',
              color: 'var(--button-fg)',
              border: '1px solid var(--border-color)',
              cursor: 'pointer',
            }}
            disabled={!isSupported || parts.length === 0}
            title="Cycle through parts (click to assign next part)"
          >
            Part: {getAssignedPartName()}
          </button>
        </div>
      </div>

      {/* Octave controls */}
      <div style={{ 
        display: 'flex', 
        justifyContent: 'center', 
        gap: '4px',
        padding: '4px',
        borderBottom: '1px solid var(--border-color)',
      }}>
        <button
          onClick={() => setOctaveOffset(prev => Math.max(0, prev - 1))}
          disabled={octaveOffset <= 0}
          style={{
            fontSize: '10px',
            padding: '2px 8px',
            background: 'var(--button-bg)',
            color: 'var(--button-fg)',
            border: '1px solid var(--border-color)',
            cursor: 'pointer',
          }}
        >
          Octave - 
        </button>
        <span style={{ fontSize: '11px', padding: '2px 8px' }}>
          Octave {octaveOffset}-{octaveOffset + 1}
        </span>
        <button
          onClick={() => setOctaveOffset(prev => Math.min(8, prev + 1))}
          disabled={octaveOffset >= 8}
          style={{
            fontSize: '10px',
            padding: '2px 8px',
            background: 'var(--button-bg)',
            color: 'var(--button-fg)',
            border: '1px solid var(--border-color)',
            cursor: 'pointer',
          }}
        >
          Octave + 
        </button>
      </div>

      {/* Keyboard display */}
      <div style={{ 
        flex: 1, 
        display: 'flex', 
        alignItems: 'flex-end',
        padding: '8px',
        overflowX: 'auto',
        gap: '2px',
        background: 'var(--editor-bg)',
      }}>
        {keys.map((key) => (
          <div
            key={`${key.note}${key.octave}`}
            onClick={() => handleKeyClick(key.midiNote)}
            style={{
              position: 'relative',
              width: key.isBlack ? `${blackKeyWidth}px` : `${whiteKeyWidth}px`,
              height: key.isBlack ? `${blackKeyHeight}px` : `${whiteKeyHeight}px`,
              background: getKeyColor(key),
              border: '1px solid #000',
              borderRadius: '2px',
              cursor: 'pointer',
              marginLeft: key.isBlack ? '-10px' : '0',
              zIndex: key.isBlack ? 2 : 1,
              display: 'flex',
              justifyContent: 'center',
              alignItems: 'flex-end',
              paddingBottom: '4px',
              boxSizing: 'border-box',
            }}
          >
            <span style={{
              fontSize: '10px',
              color: getKeyTextColor(key),
              fontWeight: 'bold',
            }}>
              {key.note}
            </span>
          </div>
        ))}
      </div>

      {/* Status */}
      <div style={{ 
        padding: '4px 8px', 
        borderTop: '1px solid var(--border-color)',
        fontSize: '10px',
        color: 'var(--text-muted)',
      }}>
        Mode: {mode} | Part: {getAssignedPartName()} | Active keys: {activeKeys.size}
      </div>
    </div>
  );
};

export default MIDIKeyboardPanel;
