import React from 'react';

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
  const [mode, setMode] = React.useState<'preview' | 'input'>('preview');
  const [assignedPart, setAssignedPart] = React.useState<string | null>('Part 1');
  const [activeKeys, setActiveKeys] = React.useState<Set<number>>(new Set());
  const [octaveOffset, setOctaveOffset] = React.useState(3);

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
    console.log(`Key pressed: MIDI ${midiNote}`);
    
    // Toggle active state
    setActiveKeys(prev => {
      const newKeys = new Set(prev);
      if (newKeys.has(midiNote)) {
        newKeys.delete(midiNote);
      } else {
        newKeys.add(midiNote);
      }
      return newKeys;
    });

    // TODO: In input mode, insert note into editor
    // TODO: In preview mode, play note via WASM chip player
    if (mode === 'preview' && assignedPart) {
      console.log(`Preview: ${midiNote} on ${assignedPart}`);
      // wasmService.previewNote(assignedPart, midiNote, 100);
    } else if (mode === 'input') {
      console.log(`Input: ${midiNote} at cursor position`);
      // insertNoteAtCursor(midiNote);
    }
  };

  const handleOctaveUp = () => {
    setOctaveOffset(prev => Math.min(prev + 1, 5));
  };

  const handleOctaveDown = () => {
    setOctaveOffset(prev => Math.max(prev - 1, 1));
  };

  // Key color based on state
  const getKeyColor = (key: MIDIKey) => {
    if (key.active) {
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
        <span style={{ fontWeight: 'bold' }}>MIDI Keyboard</span>
        <div style={{ display: 'flex', gap: '4px', alignItems: 'center' }}>
          <select
            value={mode}
            onChange={(e) => setMode(e.target.value as 'preview' | 'input')}
            style={{
              fontSize: '10px',
              padding: '2px',
              background: 'var(--input-bg)',
              color: 'var(--input-fg)',
              border: '1px solid var(--border-color)',
            }}
          >
            <option value="preview">Preview</option>
            <option value="input">Input</option>
          </select>
          <select
            value={assignedPart || ''}
            onChange={(e) => setAssignedPart(e.target.value || null)}
            style={{
              fontSize: '10px',
              padding: '2px',
              background: 'var(--input-bg)',
              color: 'var(--input-fg)',
              border: '1px solid var(--border-color)',
            }}
          >
            <option value="">No Part</option>
            <option value="Part 1">Part 1</option>
            <option value="Part 2">Part 2</option>
            <option value="Part 3">Part 3</option>
          </select>
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
          onClick={handleOctaveDown}
          disabled={octaveOffset <= 1}
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
          onClick={handleOctaveUp}
          disabled={octaveOffset >= 5}
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
        Mode: {mode} | Part: {assignedPart || 'None'} | Active keys: {activeKeys.size}
      </div>
    </div>
  );
};

export default MIDIKeyboardPanel;
