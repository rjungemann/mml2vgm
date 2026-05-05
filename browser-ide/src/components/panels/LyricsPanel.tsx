import React, { useEffect, useCallback } from 'react';

interface LyricLine {
  text: string;
  time: number; // Time in seconds
  active: boolean;
  type?: 'section' | 'line' | 'empty';
}

interface LyricsPanelProps {
  documentId?: string;
  documentContent?: string;
  currentTime?: number; // Current playback time in seconds
}

const LyricsPanel: React.FC<LyricsPanelProps> = ({
  documentId,
  documentContent = '',
  currentTime = 0,
}) => {
  // Parse lyrics from MML \ly commands
  const parseLyricsFromMML = useCallback((content: string): LyricLine[] => {
    const lines: LyricLine[] = [];
    const lyricPattern = /\\ly\s+([\d.]+)\s+(.+?)\s*$/gm;
    const sectionPattern = /^\[(.+?)\]\s*$/gm;
    
    let match;
    let currentSection: string | null = null;
    let lastTime = 0;
    
    // First, find all \ly commands and parse them
    const lyMatches: { time: number; text: string }[] = [];
    while ((match = lyricPattern.exec(content)) !== null) {
      const time = parseFloat(match[1]);
      const text = match[2].trim();
      lyMatches.push({ time, text });
    }
    
    // Also check for section markers in the content
    const contentLines = content.split('\n');
    
    // If we found \ly commands, use them
    if (lyMatches.length > 0) {
      for (const ly of lyMatches) {
        // Check if this looks like a section marker
        if (ly.text.startsWith('[') && ly.text.endsWith(']')) {
          lines.push({
            text: ly.text,
            time: ly.time,
            active: false,
            type: 'section',
          });
          currentSection = ly.text;
        } else {
          lines.push({
            text: ly.text,
            time: ly.time,
            active: false,
            type: 'line',
          });
        }
        lastTime = ly.time;
      }
    } else {
      // Fallback: try to parse section markers from content
      // Look for [Verse], [Chorus], etc.
      for (const line of contentLines) {
        const trimmed = line.trim();
        
        // Skip empty lines and MML commands
        if (!trimmed || trimmed.startsWith(';') || trimmed.startsWith('@') || 
            trimmed.startsWith('v') || trimmed.startsWith('o') || 
            trimmed.startsWith('l') || trimmed.startsWith('t') ||
            trimmed.startsWith('q')) {
          continue;
        }
        
        // Check for section markers
        const sectionMatch = trimmed.match(/^\[(.+?)\]\s*$/);
        if (sectionMatch) {
          lines.push({
            text: trimmed,
            time: lastTime,
            active: false,
            type: 'section',
          });
          currentSection = trimmed;
          lastTime += 2; // Add some time for sections
          continue;
        }
        
        // If we're in a section that looks like lyrics, add the line
        if (currentSection && trimmed.length > 0 && trimmed.length < 100) {
          lines.push({
            text: trimmed,
            time: lastTime,
            active: false,
            type: 'line',
          });
          lastTime += 2;
        }
      }
    }
    
    // If we have no lyrics, return mock data
    if (lines.length === 0 && documentContent.includes('\\ly')) {
      // Try one more time with different pattern
      const altPattern = /\\ly\s+"([^"]+)"/g;
      let altMatch;
      while ((altMatch = altPattern.exec(content)) !== null) {
        lines.push({
          text: altMatch[1],
          time: lastTime,
          active: false,
          type: 'line',
        });
        lastTime += 2;
      }
    }
    
    return lines.length > 0 ? lines : [
      { text: '[No lyrics found]', time: 0, active: false, type: 'section' },
      { text: 'Add \\ly commands to your MML', time: 0, active: false, type: 'line' },
    ];
  }, []);

  // Parse lyrics from document content
  const [lyrics, setLyrics] = React.useState<LyricLine[]>(() => {
    return parseLyricsFromMML(documentContent);
  });

  // Update lyrics when document content changes
  useEffect(() => {
    setLyrics(parseLyricsFromMML(documentContent));
  }, [documentContent, parseLyricsFromMML]);

  // Use the provided currentTime, or fall back to internal state
  const [internalCurrentTime, setInternalCurrentTime] = React.useState(0);
  const displayTime = currentTime ?? internalCurrentTime;
  const [fontSize, setFontSize] = React.useState(14);
  const [isAutoScroll, setIsAutoScroll] = React.useState(true);

  // No need for internal simulation when connected to audio player

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
