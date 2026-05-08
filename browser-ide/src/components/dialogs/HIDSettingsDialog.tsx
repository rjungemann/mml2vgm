import React, { useState, useEffect, useCallback } from 'react';
import Modal from '@/components/Modal';
import { hidService, type HIDServiceState } from '@/services/hidService';
import type { HIDSettings } from '@/types';

interface HIDSettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
  settings: HIDSettings;
  onSave: (updates: Partial<HIDSettings>) => void;
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

const inputStyle: React.CSSProperties = {
  ...{ width: '80px', padding: '5px 8px', background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', borderRadius: '4px', color: 'var(--fg-primary)', fontSize: '13px' },
};

const labelStyle: React.CSSProperties = {
  display: 'block',
  marginBottom: '4px',
  fontSize: '12px',
  color: 'var(--fg-muted)',
};

const rowStyle: React.CSSProperties = { marginBottom: '14px' };

const HIDSettingsDialog: React.FC<HIDSettingsDialogProps> = ({
  isOpen,
  onClose,
  settings,
  onSave,
}) => {
  const [local, setLocal] = useState<HIDSettings>(settings);
  const [hidState, setHidState] = useState<HIDServiceState>(hidService.getState());
  const [status, setStatus] = useState<string | null>(null);
  const [isBusy, setIsBusy] = useState(false);

  useEffect(() => {
    if (!isOpen) return;
    setLocal(settings);
    setHidState(hidService.getState());
    setStatus(null);

    const handleState = (s: HIDServiceState) => setHidState(s);
    hidService.addStateListener(handleState);
    return () => hidService.removeStateListener(handleState);
  }, [isOpen, settings]);

  // Push config changes into the live service immediately so the user can test.
  useEffect(() => {
    hidService.setReportFormat(local.reportFormat);
    hidService.setReportId(local.reportId);
    hidService.setByteOffset(local.byteOffset);
  }, [local.reportFormat, local.reportId, local.byteOffset]);

  const handleRequestDevice = useCallback(async () => {
    setIsBusy(true);
    setStatus(null);
    const granted = await hidService.requestDevice();
    setIsBusy(false);
    if (granted.length > 0) {
      hidService.enable();
      setLocal((s) => ({ ...s, enabled: true }));
      setStatus(`${granted.length} device(s) connected.`);
    } else {
      setStatus('No device selected.');
    }
  }, []);

  const handleForget = useCallback(async (vendorId: number, productId: number) => {
    await hidService.forgetDevice(vendorId, productId);
    setStatus('Device disconnected and permission revoked.');
  }, []);

  const handleSave = useCallback(() => {
    onSave(local);
    onClose();
  }, [local, onSave, onClose]);

  if (!hidService.isSupported()) {
    return (
      <Modal
        isOpen={isOpen}
        onClose={onClose}
        title="HID MIDI Controller Settings"
        footer={<button className="button primary" onClick={onClose}>Close</button>}
      >
        <p style={{ color: 'var(--fg-muted)', fontSize: '13px', lineHeight: 1.6 }}>
          <strong>Web HID API is not available</strong> in this browser.
        </p>
        <p style={{ color: 'var(--fg-muted)', fontSize: '12px', lineHeight: 1.6, marginTop: '8px' }}>
          HID MIDI support requires <strong>Chrome 89+</strong> or <strong>Edge 89+</strong> on
          desktop. Firefox and Safari do not support this API.
        </p>
        <p style={{ color: 'var(--fg-muted)', fontSize: '12px', lineHeight: 1.6, marginTop: '8px' }}>
          If your MIDI controller is a standard USB MIDI class device, use{' '}
          <strong>Tools → MIDI Settings</strong> instead (Web MIDI API, wider browser support).
        </p>
      </Modal>
    );
  }

  const connectedDevices = hidState.connectedDevices;

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="HID MIDI Controller Settings (Experimental)"
      width={480}
      footer={
        <>
          <button className="button" onClick={onClose} disabled={isBusy}>Cancel</button>
          <button className="button primary" onClick={handleSave} disabled={isBusy}>Save</button>
        </>
      }
    >
      {/* Enable toggle */}
      <div style={rowStyle}>
        <label style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '13px', cursor: 'pointer' }}>
          <input
            type="checkbox"
            checked={local.enabled}
            onChange={(e) => {
              setLocal((s) => ({ ...s, enabled: e.target.checked }));
              if (e.target.checked) hidService.enable(); else hidService.disable();
            }}
          />
          Enable HID MIDI input
        </label>
        <p style={{ margin: '4px 0 0 20px', fontSize: '11px', color: 'var(--fg-muted)' }}>
          When enabled, note events from connected HID devices feed into the MIDI keyboard
          panel and note-input mode, identical to Web MIDI input.
        </p>
      </div>

      {/* Device picker */}
      <div style={rowStyle}>
        <label style={labelStyle}>Connected HID Devices</label>
        {connectedDevices.length === 0 ? (
          <p style={{ fontSize: '12px', color: 'var(--fg-muted)', margin: '0 0 8px' }}>
            No devices connected. Click <em>Add Device</em> to open the OS picker.
          </p>
        ) : (
          <ul style={{ margin: '0 0 8px', padding: 0, listStyle: 'none' }}>
            {connectedDevices.map((d) => (
              <li
                key={`${d.vendorId}:${d.productId}`}
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                  padding: '4px 8px',
                  background: 'var(--bg-tertiary)',
                  borderRadius: '4px',
                  marginBottom: '4px',
                  fontSize: '12px',
                }}
              >
                <span>
                  {d.productName}{' '}
                  <span style={{ color: 'var(--fg-muted)' }}>
                    ({d.vendorId.toString(16).padStart(4, '0')}:{d.productId.toString(16).padStart(4, '0')})
                  </span>
                </span>
                <button
                  className="button small danger"
                  onClick={() => handleForget(d.vendorId, d.productId)}
                  style={{ marginLeft: '8px', fontSize: '11px', padding: '2px 6px' }}
                >
                  Forget
                </button>
              </li>
            ))}
          </ul>
        )}
        <button
          className="button"
          onClick={handleRequestDevice}
          disabled={isBusy}
        >
          {isBusy ? 'Opening picker…' : 'Add Device…'}
        </button>
        {status && (
          <p style={{ margin: '6px 0 0', fontSize: '12px', color: 'var(--fg-muted)', fontStyle: 'italic' }}>
            {status}
          </p>
        )}
      </div>

      {/* Report format */}
      <div style={rowStyle}>
        <label style={labelStyle}>Report Format</label>
        <select
          style={selectStyle}
          value={local.reportFormat}
          onChange={(e) =>
            setLocal((s) => ({ ...s, reportFormat: e.target.value as HIDSettings['reportFormat'] }))
          }
        >
          <option value="usb-midi-class">USB MIDI Class (4-byte packets) — most common</option>
          <option value="raw-scan">Raw scan (search for MIDI status bytes)</option>
        </select>
        <p style={{ margin: '4px 0 0', fontSize: '11px', color: 'var(--fg-muted)' }}>
          {local.reportFormat === 'usb-midi-class'
            ? 'Expects [cable|CIN, status, data1, data2] groups. Works for most HID MIDI adapters.'
            : 'Scans each byte for valid MIDI status bytes. Use when USB MIDI Class produces no output.'}
        </p>
      </div>

      {/* Report ID and byte offset */}
      <div style={{ ...rowStyle, display: 'flex', gap: '16px' }}>
        <div style={{ flex: 1 }}>
          <label style={labelStyle}>Report ID Filter</label>
          <input
            style={inputStyle}
            type="number"
            min={0}
            max={255}
            placeholder="Any"
            value={local.reportId ?? ''}
            onChange={(e) => {
              const val = e.target.value === '' ? null : parseInt(e.target.value, 10);
              setLocal((s) => ({ ...s, reportId: val }));
            }}
          />
          <p style={{ margin: '4px 0 0', fontSize: '11px', color: 'var(--fg-muted)' }}>
            Leave blank to accept all report IDs.
          </p>
        </div>
        <div style={{ flex: 1 }}>
          <label style={labelStyle}>Byte Offset</label>
          <input
            style={inputStyle}
            type="number"
            min={0}
            max={63}
            value={local.byteOffset}
            onChange={(e) =>
              setLocal((s) => ({ ...s, byteOffset: Math.max(0, parseInt(e.target.value, 10) || 0) }))
            }
          />
          <p style={{ margin: '4px 0 0', fontSize: '11px', color: 'var(--fg-muted)' }}>
            Skip leading bytes (e.g. 1 if device prepends a report ID byte).
          </p>
        </div>
      </div>

      {/* Auto-reconnect */}
      <div style={rowStyle}>
        <label style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '13px', cursor: 'pointer' }}>
          <input
            type="checkbox"
            checked={local.autoReconnect}
            onChange={(e) => setLocal((s) => ({ ...s, autoReconnect: e.target.checked }))}
          />
          Auto-reconnect to previously granted devices on startup
        </label>
      </div>

      {/* Raw report debug */}
      {hidState.lastRawReport && (
        <div style={{ marginTop: '4px' }}>
          <label style={labelStyle}>Last Raw Report (hex)</label>
          <code
            style={{
              display: 'block',
              padding: '6px 8px',
              background: 'var(--bg-tertiary)',
              borderRadius: '4px',
              fontSize: '11px',
              fontFamily: 'monospace',
              color: 'var(--fg-secondary, var(--fg-primary))',
              wordBreak: 'break-all',
            }}
          >
            {hidState.lastRawReport.map((b) => b.toString(16).padStart(2, '0')).join(' ')}
          </code>
          <p style={{ margin: '4px 0 0', fontSize: '11px', color: 'var(--fg-muted)' }}>
            Use this to identify the report structure when tuning offset and report format.
          </p>
        </div>
      )}

      <div style={{ marginTop: '14px', padding: '10px 12px', background: 'var(--bg-tertiary)', borderRadius: '4px', fontSize: '11px', color: 'var(--fg-muted)', lineHeight: 1.6 }}>
        <strong>Tip:</strong> Most controllers that work here also work with the standard{' '}
        <strong>Web MIDI API</strong> (Tools → MIDI Settings). Try that first — it has wider
        browser support. Use HID only for controllers that don&apos;t show up as MIDI devices.
      </div>
    </Modal>
  );
};

export default HIDSettingsDialog;
