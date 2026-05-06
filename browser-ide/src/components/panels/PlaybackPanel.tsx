import React, { useState, useEffect, useRef } from 'react';
import { audioService } from '@/services/audioService';
import type { AudioStatus } from '@/services/audioService';
import { useSessionStorageState } from '@/utils/useSessionStorageState';

interface PlaybackPanelProps {
  // Props can be used to pass document data or compile results
  compiledData?: Uint8Array;
  chips?: string[];
}

const PlaybackPanel: React.FC<PlaybackPanelProps> = ({
  compiledData,
  chips = ['YM2608', 'SN76489'],
}) => {
  // Get initial state from audio service
  const [status, setStatus] = useState<AudioStatus>(audioService.getStatus());
  const [volume, setVolume] = useSessionStorageState<number>('mml2vgm:playback:volume', Math.round(audioService.getVolume() * 100));
  const [loop, setLoop] = useSessionStorageState<boolean>('mml2vgm:playback:loop', audioService.isLooping());
  const [duration, setDuration] = useState(0);
  const [error, setError] = useState<string | null>(null);

  const timeUpdateListener = useRef<() => void>();
  const playListener = useRef<() => void>();
  const pauseListener = useRef<() => void>();
  const stopListener = useRef<() => void>();
  const errorListener = useRef<(e: Error) => void>();

  // Initialize with audio service state
  useEffect(() => {
    // Add listeners for audio service events
    timeUpdateListener.current = () => {
      setStatus(audioService.getStatus());
    };
    
    playListener.current = () => {
      setStatus(audioService.getStatus());
    };
    
    pauseListener.current = () => {
      setStatus(audioService.getStatus());
    };
    
    stopListener.current = () => {
      setStatus(audioService.getStatus());
    };
    
    errorListener.current = (e: Error) => {
      setError(e.message);
    };

    audioService.addEventListener({
      onPlay: playListener.current,
      onPause: pauseListener.current,
      onStop: stopListener.current,
      onTimeUpdate: timeUpdateListener.current,
      onError: errorListener.current,
    });

    // Initialize audio service if not already done
    audioService.init().catch(console.error);

    return () => {
      audioService.removeEventListener({
        onPlay: playListener.current,
        onPause: pauseListener.current,
        onStop: stopListener.current,
        onTimeUpdate: timeUpdateListener.current,
        onError: errorListener.current,
      });
    };
  }, []);

  // Sync volume with audio service
  useEffect(() => {
    audioService.setVolume(volume / 100);
  }, [volume]);

  // Sync loop with audio service
  useEffect(() => {
    audioService.setLoop(loop);
  }, [loop]);

  // Format time display (milliseconds to MM:SS)
  const formatTime = (milliseconds: number): string => {
    const seconds = Math.floor(milliseconds / 1000);
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  // Calculate progress percentage
  const progressPercent = duration > 0 ? (status.currentTime / duration) * 100 : 0;

  // Handle play/pause toggle
  const handlePlayPause = async () => {
    setError(null);
    
    try {
      if (status.isPlaying && !status.isPaused) {
        // Pause
        audioService.pause();
      } else if (status.isPaused) {
        // Resume
        audioService.resume();
      } else {
        // Play
        if (compiledData) {
          await audioService.playVGM(compiledData, { 
            chips: chips as any[], 
            volume: volume / 100 
          });
        } else {
          console.log('No compiled data to play');
        }
      }
    } catch (error) {
      console.error('Playback error:', error);
      setError(error instanceof Error ? error.message : String(error));
    }
  };

  // Handle stop
  const handleStop = () => {
    audioService.stop();
    setError(null);
  };

  // Handle seek
  const handleSeek = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!duration) return;
    
    const rect = e.currentTarget.getBoundingClientRect();
    const clickPosition = e.clientX - rect.left;
    const percent = clickPosition / rect.width;
    const newTime = percent * duration;
    
    audioService.seek(newTime);
    setStatus(audioService.getStatus());
  };

  // Get button label based on state
  const getPlayButtonLabel = () => {
    if (status.isPaused) return '▶';
    if (status.isPlaying) return '⏸';
    return '▶';
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', padding: '4px' }}>
      {/* Playback controls */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '4px', marginBottom: '4px' }}>
        <button
          style={{
            padding: '4px 8px',
            fontSize: '12px',
            background: 'var(--button-bg)',
            color: 'var(--button-fg)',
            border: '1px solid var(--border-color)',
            cursor: 'pointer',
            minWidth: '40px',
          }}
          onClick={handlePlayPause}
          title={status.isPaused ? 'Resume' : status.isPlaying ? 'Pause' : 'Play'}
        >
          {getPlayButtonLabel()}
        </button>
        
        <button
          style={{
            padding: '4px 8px',
            fontSize: '12px',
            background: 'var(--button-bg)',
            color: 'var(--button-fg)',
            border: '1px solid var(--border-color)',
            cursor: 'pointer',
            minWidth: '40px',
          }}
          onClick={handleStop}
          disabled={!status.isPlaying && !status.isPaused}
          title="Stop"
        >
          ⏹
        </button>

        <div style={{ width: '8px' }} />
        
        {/* Volume control */}
        <span style={{ fontSize: '11px' }}>Vol:</span>
        <input
          type="range"
          min={0}
          max={100}
          value={volume}
          onChange={(e) => setVolume(Number(e.target.value))}
          style={{ width: '60px' }}
        />
        <span style={{ fontSize: '11px', marginLeft: '4px' }}>{volume}%</span>

        <div style={{ flex: 1 }} />
        
        {/* Loop toggle */}
        <button
          style={{
            padding: '2px 6px',
            fontSize: '10px',
            background: loop ? 'var(--button-active-bg)' : 'var(--button-bg)',
            color: loop ? 'var(--button-active-fg)' : 'var(--button-fg)',
            border: '1px solid var(--border-color)',
            cursor: 'pointer',
          }}
          onClick={() => setLoop(!loop)}
        >
          Loop: {loop ? 'ON' : 'OFF'}
        </button>
      </div>

      {/* Timeline */}
      <div style={{ marginTop: '4px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '4px' }}>
          <span style={{ fontSize: '11px', color: 'var(--text-muted)', minWidth: '40px' }}>
            {formatTime(status.currentTime)}
          </span>
          <div 
            style={{ 
              flex: 1, 
              height: '4px', 
              backgroundColor: 'var(--bg-tertiary)', 
              borderRadius: '2px', 
              cursor: 'pointer',
              position: 'relative',
            }}
            onClick={handleSeek}
          >
            <div
              style={{
                position: 'absolute',
                left: 0,
                top: 0,
                height: '100%',
                width: `${progressPercent}%`,
                backgroundColor: 'var(--accent-primary)',
                borderRadius: '2px',
              }}
            />
          </div>
          <span style={{ fontSize: '11px', color: 'var(--text-muted)', minWidth: '40px' }}>
            {formatTime(duration)}
          </span>
        </div>
      </div>

      {/* Playback info */}
      <div 
        style={{
          marginTop: '8px',
          padding: '4px',
          backgroundColor: 'var(--bg-tertiary)',
          borderRadius: '3px',
          fontSize: '11px',
        }}
      >
        <div>Status: {status.isPlaying ? (status.isPaused ? 'Paused' : 'Playing') : 'Stopped'}</div>
        <div>Position: {formatTime(status.currentTime)} / {formatTime(duration)}</div>
        <div>Sample Rate: {status.sampleRate} Hz</div>
        <div>Chips: {status.chips.length > 0 ? status.chips.join(', ') : 'None'}</div>
        {error && (
          <div style={{ color: 'var(--text-error)' }}>Error: {error}</div>
        )}
      </div>
    </div>
  );
};

export default PlaybackPanel;
