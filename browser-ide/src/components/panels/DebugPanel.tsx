import React from 'react';

interface DebugMessage {
  timestamp: string;
  type: 'info' | 'warning' | 'error' | 'log';
  source: string;
  message: string;
  data?: any;
}

const DebugPanel: React.FC = () => {
  const [messages, setMessages] = React.useState<DebugMessage[]>([
    { timestamp: new Date().toISOString(), type: 'info', source: 'System', message: 'Debug panel initialized' },
  ]);
  const [filter, setFilter] = React.useState<'all' | 'info' | 'warning' | 'error' | 'log'>('all');
  const [autoScroll, setAutoScroll] = React.useState(true);
  const messagesEndRef = React.useRef<HTMLDivElement>(null);

  // Mock debug messages
  React.useEffect(() => {
    // Add some mock messages
    const timer = setTimeout(() => {
      setMessages(prev => [
        ...prev,
        { timestamp: new Date().toISOString(), type: 'log', source: 'WASM', message: 'Module loaded successfully' },
        { timestamp: new Date().toISOString(), type: 'log', source: 'Audio', message: 'AudioContext created' },
        { timestamp: new Date().toISOString(), type: 'info', source: 'Editor', message: 'Monaco initialized' },
      ]);
    }, 100);
    
    return () => clearTimeout(timer);
  }, []);

  // Auto-scroll to bottom
  React.useEffect(() => {
    if (autoScroll && messagesEndRef.current) {
      messagesEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [messages, autoScroll]);

  const addMessage = (type: DebugMessage['type'], source: string, message: string, data?: any) => {
    setMessages(prev => [
      ...prev,
      { timestamp: new Date().toISOString(), type, source, message, data },
    ]);
  };

  const clearMessages = () => {
    setMessages([]);
    addMessage('info', 'System', 'Debug panel cleared');
  };

  const filteredMessages = messages.filter(msg => {
    if (filter === 'all') return true;
    return msg.type === filter;
  });

  // Color for message type
  const getMessageColor = (type: DebugMessage['type']) => {
    switch (type) {
      case 'error': return '#ff6b6b';
      case 'warning': return '#f0ad4e';
      case 'info': return '#5bc0de';
      case 'log': return '#9d9d9d';
      default: return '#ffffff';
    }
  };

  // Background for message type
  const getMessageBg = (type: DebugMessage['type']) => {
    switch (type) {
      case 'error': return 'rgba(255, 107, 107, 0.1)';
      case 'warning': return 'rgba(240, 173, 78, 0.1)';
      case 'info': return 'rgba(91, 192, 222, 0.1)';
      case 'log': return 'rgba(157, 157, 157, 0.1)';
      default: return 'transparent';
    }
  };

  // Format timestamp
  const formatTimestamp = (isoString: string) => {
    const date = new Date(isoString);
    return date.toLocaleTimeString();
  };

  // Format data for display
  const formatData = (data: any) => {
    if (data === undefined) return '';
    if (typeof data === 'string') return data;
    return JSON.stringify(data, null, 2);
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
        <span style={{ fontWeight: 'bold' }}>Debug</span>
        <div style={{ display: 'flex', gap: '4px', alignItems: 'center' }}>
          <select
            value={filter}
            onChange={(e) => setFilter(e.target.value as typeof filter)}
            style={{
              fontSize: '10px',
              padding: '2px',
              background: 'var(--input-bg)',
              color: 'var(--input-fg)',
              border: '1px solid var(--border-color)',
            }}
          >
            <option value="all">All</option>
            <option value="info">Info</option>
            <option value="warning">Warning</option>
            <option value="error">Error</option>
            <option value="log">Log</option>
          </select>
          <button
            onClick={clearMessages}
            style={{
              fontSize: '10px',
              padding: '2px 4px',
              background: 'var(--button-bg)',
              color: 'var(--button-fg)',
              border: '1px solid var(--border-color)',
              cursor: 'pointer',
            }}
          >
            Clear
          </button>
          <label style={{ display: 'flex', alignItems: 'center', gap: '2px' }}>
            <input
              type="checkbox"
              checked={autoScroll}
              onChange={(e) => setAutoScroll(e.target.checked)}
              style={{ margin: 0 }}
            />
            <span style={{ fontSize: '10px' }}>Auto</span>
          </label>
        </div>
      </div>

      {/* Messages list */}
      <div style={{ 
        flex: 1, 
        overflowY: 'auto', 
        padding: '4px',
        fontSize: '10px',
      }}>
        {filteredMessages.length === 0 ? (
          <div style={{ 
            color: 'var(--text-muted)', 
            fontStyle: 'italic',
            padding: '20px',
            textAlign: 'center',
          }}>
            No debug messages
          </div>
        ) : (
          filteredMessages.map((msg, index) => (
            <div
              key={index}
              style={{
                padding: '4px',
                borderBottom: '1px solid var(--border-color)',
                color: getMessageColor(msg.type),
                background: getMessageBg(msg.type),
              }}
            >
              <div style={{ display: 'flex', gap: '8px' }}>
                <span style={{ color: 'var(--text-muted)', minWidth: '60px' }}>
                  [{formatTimestamp(msg.timestamp)}]
                </span>
                <span style={{ 
                  fontWeight: 'bold', 
                  minWidth: '60px',
                  textTransform: 'uppercase',
                }}>
                  {msg.type}
                </span>
                <span style={{ 
                  color: 'var(--text-muted)', 
                  minWidth: '80px',
                }}>
                  {msg.source}:
                </span>
                <span style={{ flex: 1 }}>{msg.message}</span>
              </div>
              {msg.data && (
                <pre style={{
                  marginTop: '4px',
                  marginLeft: '150px',
                  padding: '4px',
                  background: 'var(--editor-bg)',
                  borderRadius: '2px',
                  overflowX: 'auto',
                  whiteSpace: 'pre-wrap',
                  wordBreak: 'break-all',
                }}>
                  {formatData(msg.data)}
                </pre>
              )}
            </div>
          ))
        )}
        <div ref={messagesEndRef} />
      </div>

      {/* Actions */}
      <div style={{ 
        display: 'flex', 
        gap: '4px',
        padding: '4px 8px',
        borderTop: '1px solid var(--border-color)',
      }}>
        <button
          onClick={() => addMessage('log', 'Test', 'Test log message')}
          style={{
            flex: 1,
            fontSize: '10px',
            padding: '4px',
            background: 'var(--button-bg)',
            color: 'var(--button-fg)',
            border: '1px solid var(--border-color)',
            cursor: 'pointer',
          }}
        >
          Test Log
        </button>
        <button
          onClick={() => addMessage('info', 'Test', 'Test info message')}
          style={{
            flex: 1,
            fontSize: '10px',
            padding: '4px',
            background: 'var(--button-bg)',
            color: 'var(--button-fg)',
            border: '1px solid var(--border-color)',
            cursor: 'pointer',
          }}
        >
          Test Info
        </button>
        <button
          onClick={() => addMessage('warning', 'Test', 'Test warning message')}
          style={{
            flex: 1,
            fontSize: '10px',
            padding: '4px',
            background: 'var(--button-bg)',
            color: 'var(--button-fg)',
            border: '1px solid var(--border-color)',
            cursor: 'pointer',
          }}
        >
          Test Warn
        </button>
      </div>

      {/* Status */}
      <div style={{ 
        padding: '4px 8px', 
        borderTop: '1px solid var(--border-color)',
        fontSize: '10px',
        color: 'var(--text-muted)',
      }}>
        Total: {messages.length} | Filtered: {filteredMessages.length} | Type: {filter}
      </div>
    </div>
  );
};

export default DebugPanel;
