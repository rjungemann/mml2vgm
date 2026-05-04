import React from 'react';

interface LyricLine {
  text: string;
  time: number; // Time in seconds
  active: boolean;
}

const LyricsPanel: React.FC = () => {
  // Mock lyrics data - TODO: Parse from MML \ly commands
  const [lyrics, setLyrics] = React.useState<LyricLine[]>([
    { text: '[Verse 1]', time: 0, active: false },
    { text: 'This is the first line of lyrics', time: 2, active: false },
    { text: 'Second line of the verse', time: 4, active: false },
    { text: '', time: 6, active: false },
    { text: '[Chorus]', time: 8, active: false },
    { text: 'This is the chorus line one', time: 10, active: false },
    { text: 'Chorus line two goes here', time: 12, active: false },
  ]);

  const [currentTime, setCurrentTime] = React.useState(0);
  const [fontSize, setFontSize] = React.useState(14);
  const [isAutoScroll, setIsAutoScroll] = React.useState(true);

  // Simulate playback position updates
  React.useEffect(() => {
    // TODO: Connect to actual audio player position
    // This is a mock simulation
    const interval = setInterval(() => {
      // Just demo - in real implementation, this would come from audio player
      // setCurrentTime(prev => prev + 0.1);
    }, 100);
    
    return () => clearInterval(interval);
  }, []);

  // Update active lyric based on current time
  React.useEffect(() => {
    const newLyrics = lyrics.map(lyric => ({
      ...lyric,
      active: currentTime >= lyric.time && currentTime < (lyric.time + 2),
    }));
    setLyrics(newLyrics);
    
    // Auto-scroll to active lyric
    if (isAutoScroll) {
      const activeIndex = newLyrics.findIndex(l => l.active);
      if (activeIndex >= 0) {
        const element = document.getElementById(`lyric-${activeIndex}`);
        element?.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }
    }
  }, [currentTime, isAutoScroll, lyrics]);

  const handleFontSizeChange = (delta: number) => {
    setFontSize(prev => Math.max(8, Math.min(24, prev + delta)));
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      {/* Header */}
      <div style={{ 
        display: 'flex', 
        justifyContent: 'space-between', 
        alignItems: 'center',
        padding: '4px 8px',
        borderBottom: '1px solid var(--border-color)',
        fontSize: '11px',
      }}>
        <span style={{ fontWeight: 'bold' }}>Lyrics</span>
        <div style={{ display: 'flex', gap: '4px', alignItems: 'center' }}>
          <button
            onClick={() => handleFontSizeChange(-2)}
            style={{
              fontSize: '10px',
              padding: '2px 4px',
              background: 'var(--button-bg)',
              color: 'var(--button-fg)',
              border: '1px solid var(--border-color)',
              cursor: 'pointer',
            }}
          >
            -
          </button>
          <span style={{ fontSize: '11px' }}>{fontSize}px</span>
          <button
            onClick={() => handleFontSizeChange(2)}
            style={{
              fontSize: '10px',
              padding: '2px 4px',
              background: 'var(--button-bg)',
              color: 'var(--button-fg)',
              border: '1px solid var(--border-color)',
              cursor: 'pointer',
            }}
          >
            +
          </button>
          <label style={{ display: 'flex', alignItems: 'center', gap: '2px' }}>
            <input
              type="checkbox"
              checked={isAutoScroll}
              onChange={(e) => setIsAutoScroll(e.target.checked)}
              style={{ margin: 0 }}
            />
            <span style={{ fontSize: '10px' }}>Auto</span>
          </label>
        </div>
      </div>

      {/* Lyrics content */}
      <div style={{ 
        flex: 1, 
        overflowY: 'auto', 
        padding: '8px',
        textAlign: 'center',
      }}>
        {lyrics.length === 0 ? (
          <div style={{ 
            color: 'var(--text-muted)', 
            fontStyle: 'italic',
            padding: '20px',
          }}>
            No lyrics available
          </div>
        ) : (
          lyrics.map((lyric, index) => (
            <div
              key={index}
              id={`lyric-${index}`}
              style={{
                fontSize: `${fontSize}px`,
                padding: '4px 0',
                color: lyric.active ? 'var(--text-highlight)' : 'var(--text)',
                background: lyric.active ? 'var(--selection-bg)' : 'transparent',
                transition: 'all 0.2s ease',
              }}
            >
              {lyric.text || <span style={{ color: 'var(--text-muted)' }}>...</span>}
            </div>
          ))
        )}
      </div>

      {/* Footer with current time */}
      <div style={{ 
        padding: '4px 8px', 
        borderTop: '1px solid var(--border-color)',
        fontSize: '10px',
        color: 'var(--text-muted)',
        textAlign: 'right',
      }}>
        {currentTime.toFixed(1)}s
      </div>
    </div>
  );
};

export default LyricsPanel;
