import React, { useState, useEffect, useCallback } from 'react';
import Modal from '@/components/Modal';
import { serialService, type SerialServiceState, type SerialConnectOptions } from '@/services/serialService';
import type { SerialSettings } from '@/types';

interface SerialSettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
  settings: SerialSettings;
  onSave: (updates: Partial<SerialSettings>) => void;
}

const selectStyle: React.CSSProperties = {
  width: '100%',
  padding: '5px 8px',
  background: 'var(--bg-tertiary)',
  border: '1px solid var(--border-color)',
  borderRadius: '4px',
  color: 'var(--fg-primary)',
  fontSize: '13px',
};

const labelStyle: React.CSSProperties = {
  display: 'block',
  marginBottom: '4px',
  fontSize: '12px',
  color: 'var(--fg-muted)',
};

const rowStyle: React.CSSProperties = { marginBottom: '14px' };

const BAUD_RATES = [9600, 38400, 57600, 115200];
const PROTOCOLS: Array<{ value: SerialSettings['protocol']; label: string; description: string }> = [
  {
    value: 'gimic',
    label: 'GIMIC',
    description: 'GIMIC OPN2/OPNA/OPM modules (FTDI USB-serial bridge)',
  },
  {
    value: 'scci-raw',
    label: 'SCCI Raw',
    description: 'Homebrew SCCI adapters — 3-byte framing (addr, data, 0x00)',
  },
  {
    value: 'generic',
    label: 'Generic',
    description: 'Generic 2-byte framing (addr, data) for custom adapters',
  },
];

const STATUS_COLORS: Record<string, string> = {
  connected: 'var(--success, #4caf50)',
  disconnected: 'var(--fg-muted)',
  unsupported: 'var(--warning, #ff9800)',
};

const SerialSettingsDialog: React.FC<SerialSettingsDialogProps> = ({
  isOpen,
  onClose,
  settings,
  onSave,
}) => {
  const [localSettings, setLocalSettings] = useState<SerialSettings>(settings);
  const [serialState, setSerialState] = useState<SerialServiceState>(serialService.getState());
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [isBusy, setIsBusy] = useState(false);

  // Sync local settings and subscribe to service state changes when dialog opens.
  useEffect(() => {
    if (!isOpen) return;
    setLocalSettings(settings);
    setSerialState(serialService.getState());
    setStatusMessage(null);

    const handleState = (state: SerialServiceState) => setSerialState(state);
    serialService.addStateListener(handleState);
    return () => serialService.removeStateListener(handleState);
  }, [isOpen, settings]);

  const handleRequestPort = useCallback(async () => {
    setIsBusy(true);
    setStatusMessage(null);
    const granted = await serialService.requestPort(localSettings.protocol === 'gimic');
    setIsBusy(false);
    setStatusMessage(granted ? 'Port selected — click Connect to open it.' : 'No port selected.');
  }, [localSettings.protocol]);

  const handleConnect = useCallback(async () => {
    setIsBusy(true);
    setStatusMessage(null);
    const options: SerialConnectOptions = {
      baudRate: localSettings.baudRate,
      protocol: localSettings.protocol,
    };
    const ok = await serialService.connect(options);
    setIsBusy(false);
    setStatusMessage(ok ? 'Connected successfully.' : 'Failed to connect. Check port and baud rate.');
  }, [localSettings.baudRate, localSettings.protocol]);

  const handleDisconnect = useCallback(async () => {
    setIsBusy(true);
    await serialService.disconnect();
    setIsBusy(false);
    setStatusMessage('Disconnected.');
  }, []);

  const handleSave = useCallback(() => {
    onSave(localSettings);
    onClose();
  }, [localSettings, onSave, onClose]);

  if (!serialService.isSupported()) {
    return (
      <Modal
        isOpen={isOpen}
        onClose={onClose}
        title="Hardware Serial Settings"
        footer={<button className="button primary" onClick={onClose}>Close</button>}
      >
        <p style={{ color: 'var(--fg-muted)', fontSize: '13px', lineHeight: 1.6 }}>
          <strong>Web Serial API is not available</strong> in this browser.
        </p>
        <p style={{ color: 'var(--fg-muted)', fontSize: '12px', lineHeight: 1.6, marginTop: '8px' }}>
          Hardware access via WebSerial requires <strong>Chrome 89+</strong> or <strong>Edge 89+</strong>
          {' '}on desktop. Firefox and Safari do not support this API.
        </p>
        <p style={{ color: 'var(--fg-muted)', fontSize: '12px', lineHeight: 1.6, marginTop: '8px' }}>
          Supported devices: GIMIC OPN2/OPNA/OPM modules, SCCI-compatible adapters.
        </p>
      </Modal>
    );
  }

  const isConnected = serialState.isConnected;
  const statusColor = isConnected
    ? STATUS_COLORS.connected
    : serialState.isSupported
    ? STATUS_COLORS.disconnected
    : STATUS_COLORS.unsupported;

  const portInfo = serialState.port;

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Hardware Serial Settings (Experimental)"
      width={460}
      footer={
        <>
          <button className="button" onClick={onClose} disabled={isBusy}>Cancel</button>
          <button className="button primary" onClick={handleSave} disabled={isBusy}>Save</button>
        </>
      }
    >
      {/* Connection status */}
      <div style={{ ...rowStyle, display: 'flex', alignItems: 'center', gap: '8px' }}>
        <span
          style={{
            display: 'inline-block',
            width: '8px',
            height: '8px',
            borderRadius: '50%',
            background: statusColor,
            flexShrink: 0,
          }}
        />
        <span style={{ fontSize: '12px', color: 'var(--fg-secondary, var(--fg-primary))' }}>
          {isConnected
            ? `Connected — ${portInfo?.protocol?.toUpperCase() ?? 'GIMIC'} @ ${portInfo?.baudRate ?? '?'} baud`
            : 'Not connected'}
        </span>
      </div>

      {statusMessage && (
        <div style={{ ...rowStyle, fontSize: '12px', color: 'var(--fg-muted)', fontStyle: 'italic' }}>
          {statusMessage}
        </div>
      )}

      {/* Protocol */}
      <div style={rowStyle}>
        <label style={labelStyle}>Protocol</label>
        <select
          style={selectStyle}
          value={localSettings.protocol}
          onChange={(e) =>
            setLocalSettings((s) => ({ ...s, protocol: e.target.value as SerialSettings['protocol'] }))
          }
          disabled={isConnected || isBusy}
        >
          {PROTOCOLS.map((p) => (
            <option key={p.value} value={p.value}>
              {p.label} — {p.description}
            </option>
          ))}
        </select>
      </div>

      {/* Baud rate */}
      <div style={rowStyle}>
        <label style={labelStyle}>Baud Rate</label>
        <select
          style={selectStyle}
          value={localSettings.baudRate}
          onChange={(e) =>
            setLocalSettings((s) => ({ ...s, baudRate: Number(e.target.value) }))
          }
          disabled={isConnected || isBusy}
        >
          {BAUD_RATES.map((r) => (
            <option key={r} value={r}>
              {r.toLocaleString()}
            </option>
          ))}
        </select>
        <p style={{ margin: '4px 0 0', fontSize: '11px', color: 'var(--fg-muted)' }}>
          GIMIC modules typically use <strong>38400</strong> baud.
        </p>
      </div>

      {/* Auto-reconnect */}
      <div style={rowStyle}>
        <label style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '13px', cursor: 'pointer' }}>
          <input
            type="checkbox"
            checked={localSettings.autoReconnect}
            onChange={(e) =>
              setLocalSettings((s) => ({ ...s, autoReconnect: e.target.checked }))
            }
          />
          Auto-reconnect to previously granted port on startup
        </label>
        <p style={{ margin: '4px 0 0 20px', fontSize: '11px', color: 'var(--fg-muted)' }}>
          The browser stores granted port handles; no additional permission prompt is needed.
        </p>
      </div>

      {/* Port selection & connect */}
      <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
        <button
          className="button"
          onClick={handleRequestPort}
          disabled={isConnected || isBusy}
          title="Open the OS port picker to select a serial device"
        >
          Select Port…
        </button>
        {!isConnected ? (
          <button
            className="button primary"
            onClick={handleConnect}
            disabled={isBusy}
          >
            {isBusy ? 'Connecting…' : 'Connect'}
          </button>
        ) : (
          <button
            className="button danger"
            onClick={handleDisconnect}
            disabled={isBusy}
          >
            {isBusy ? 'Disconnecting…' : 'Disconnect'}
          </button>
        )}
      </div>

      <div style={{ marginTop: '16px', padding: '10px 12px', background: 'var(--bg-tertiary)', borderRadius: '4px', fontSize: '11px', color: 'var(--fg-muted)', lineHeight: 1.6 }}>
        <strong>Supported hardware:</strong> GIMIC OPN2, OPNA, OPM, OPL2/3 modules;
        homebrew SCCI serial adapters; DIY boards with generic 2-byte protocol.
        Plug in the device before clicking <em>Select Port</em>.
        GIMIC modules require no additional drivers on Windows 10/11 or macOS 12+.
      </div>
    </Modal>
  );
};

export default SerialSettingsDialog;
