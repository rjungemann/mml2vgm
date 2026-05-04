import React, { useState, useEffect } from 'react';
import { traceService } from '@/services/traceService';
import { partService } from '@/services/partService';
import type { PartInfo, PartWithState, SoundChip } from '@/types';

interface PartCounterPanelProps {
  // Can be connected to actual compile results
  parts?: PartInfo[];
  documentId?: string;
}

const PartCounterPanel: React.FC<PartCounterPanelProps> = ({
  parts: externalParts,
  documentId,
}) => {
  // Get parts from partService
  const [parts, setParts] = useState<PartWithState[]>([]);
  
  // Get active parts from trace service
  const [activeParts, setActiveParts] = useState<Set<string>>(new Set());
  
  // Get part service parts when document changes
  useEffect(() => {
    if (documentId) {
      // Try to get parts from partService
      const partsFromService = partService.getParts();
      if (partsFromService.length > 0) {
        setParts(partsFromService);
      } else if (externalParts) {
        setParts(externalParts as PartWithState[]);
      } else {
        // Fallback to mock data
        setParts([
          { index: 0, name: 'FM1', chip: 'YM2608' as SoundChip, channel: 0, volume: 100, pan: 0, isSolo: false, isMuted: false, isKbdAssigned: false, startPosition: { line: 1, column: 1 }, endPosition: { line: 1, column: 1 } },
          { index: 1, name: 'FM2', chip: 'YM2608' as SoundChip, channel: 1, volume: 100, pan: 0, isSolo: false, isMuted: false, isKbdAssigned: false, startPosition: { line: 1, column: 1 }, endPosition: { line: 1, column: 1 } },
          { index: 2, name: 'FM3', chip: 'YM2608' as SoundChip, channel: 2, volume: 100, pan: 0, isSolo: false, isMuted: false, isKbdAssigned: false, startPosition: { line: 1, column: 1 }, endPosition: { line: 1, column: 1 } },
          { index: 3, name: 'SSG1', chip: 'AY8910' as SoundChip, channel: 0, volume: 80, pan: 0, isSolo: false, isMuted: false, isKbdAssigned: false, startPosition: { line: 1, column: 1 }, endPosition: { line: 1, column: 1 } },
          { index: 4, name: 'SSG2', chip: 'AY8910' as SoundChip, channel: 1, volume: 80, pan: 0, isSolo: false, isMuted: false, isKbdAssigned: false, startPosition: { line: 1, column: 1 }, endPosition: { line: 1, column: 1 } },
          { index: 5, name: 'SSG3', chip: 'AY8910' as SoundChip, channel: 2, volume: 80, pan: 0, isSolo: false, isMuted: false, isKbdAssigned: false, startPosition: { line: 1, column: 1 }, endPosition: { line: 1, column: 1 } },
        ]);
      }
    } else if (externalParts) {
      setParts(externalParts as PartWithState[]);
    }
  }, [documentId, externalParts]);
  
  // Listen to partService for part changes
  useEffect(() => {
    const handlePartsUpdate = (updatedParts: PartWithState[]) => {
      setParts(updatedParts);
    };
    
    partService.addListener(handlePartsUpdate);
    
    return () => {
      partService.removeListener(handlePartsUpdate);
    };
  }, []);

  // Listen to trace service for active parts
  useEffect(() => {
    const handleTraceUpdate = () => {
      setActiveParts(traceService.getActiveParts());
    };
    
    const listener = {
      onTraceStart: handleTraceUpdate,
      onTraceStop: handleTraceUpdate,
      onPartEvent: handleTraceUpdate,
    };
    
    traceService.addEventListener(listener);
    
    // Initial update
    setActiveParts(traceService.getActiveParts());
    
    return () => {
      traceService.removeEventListener(listener);
    };
  }, []);

  // Toggle mute for a part
  const handleToggleMute = (index: number) => {
    partService.toggleMute(index);
  };

  // Toggle solo for a part
  const handleToggleSolo = (index: number) => {
    partService.toggleSolo(index);
  };

  // Get stats
  const totalParts = parts.length;
  const activePartCount = parts.filter(p => !p.isMuted).length;
  const mutedPartCount = parts.filter(p => p.isMuted).length;
  const soloPartCount = parts.filter(p => p.isSolo).length;

  // Check if a part is currently active (from trace)
  const isPartActive = (index: number) => {
    return activeParts.has(`part-${index}`);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      {/* Summary */}
      <div style={{ padding: '4px 8px', borderBottom: '1px solid var(--border-color)' }}>
        <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
          Total: {totalParts}
        </div>
        <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
          Active: {activePartCount}
        </div>
        <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
          Muted: {mutedPartCount}
        </div>
        <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
          Solo: {soloPartCount}
        </div>
        {traceService.isTracing() && (
          <div style={{ 
            fontSize: '11px', 
            color: 'var(--accent-primary)',
            marginTop: '4px',
            fontWeight: 'bold'
          }}>
            Tracing: ON
          </div>
        )}
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
              backgroundColor: isPartActive(part.index) 
                ? 'rgba(0, 255, 0, 0.15)'
                : part.isMuted 
                  ? 'var(--bg-tertiary)'
                  : 'transparent',
              borderLeft: isPartActive(part.index) 
                ? '3px solid var(--accent-primary)'
                : '3px solid transparent',
            }}
            onClick={() => handleToggleMute(part.index)}
            title={isPartActive(part.index) ? 'Currently active (playing)' : part.isMuted ? 'Muted' : 'Active'}
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
                  background: part.isMuted ? 'var(--button-active-bg)' : 'var(--button-bg)',
                  color: part.isMuted ? 'var(--button-active-fg)' : 'var(--button-fg)',
                  cursor: 'pointer',
                  fontSize: '10px',
                  borderRadius: '2px',
                }}
                onClick={(e) => {
                  e.stopPropagation();
                  handleToggleMute(part.index);
                }}
              >
                M
              </button>
            </span>
            <span style={{ marginLeft: '2px' }}>
              <button
                style={{
                  width: '18px',
                  height: '18px',
                  border: 'none',
                  background: part.isSolo ? 'var(--button-active-bg)' : 'var(--button-bg)',
                  color: part.isSolo ? 'var(--button-active-fg)' : 'var(--button-fg)',
                  cursor: 'pointer',
                  fontSize: '10px',
                  borderRadius: '2px',
                }}
                onClick={(e) => {
                  e.stopPropagation();
                  handleToggleSolo(part.index);
                }}
              >
                S
              </button>
            </span>
            <span style={{ marginLeft: '2px' }}>
              <button
                style={{
                  width: '18px',
                  height: '18px',
                  border: 'none',
                  background: part.isKbdAssigned ? 'var(--button-active-bg)' : 'var(--button-bg)',
                  color: part.isKbdAssigned ? 'var(--button-active-fg)' : 'var(--button-fg)',
                  cursor: 'pointer',
                  fontSize: '10px',
                  borderRadius: '2px',
                }}
                onClick={(e) => {
                  e.stopPropagation();
                  // Toggle KBD assignment
                  partService.assignKbd(part.index);
                }}
              >
                K
              </button>
            </span>
          </div>
        ))}
      </div>
    </div>
  );
};

export default PartCounterPanel;
