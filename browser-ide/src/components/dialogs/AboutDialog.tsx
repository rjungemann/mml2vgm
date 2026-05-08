import React from 'react';
import Modal from '@/components/Modal';

interface AboutDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

const AboutDialog: React.FC<AboutDialogProps> = ({ isOpen, onClose }) => (
  <Modal
    isOpen={isOpen}
    onClose={onClose}
    title="About mml2vgm"
    footer={
      <button className="button primary" onClick={onClose}>
        Close
      </button>
    }
  >
    <div style={{ textAlign: 'center', padding: '8px 0' }}>
      <div style={{ fontSize: '32px', marginBottom: '8px' }}>🎵</div>
      <h2 style={{ margin: '0 0 8px', fontSize: '18px', fontWeight: 600 }}>mml2vgm Browser IDE</h2>
      <p style={{ margin: '0 0 12px', color: 'var(--fg-muted, #888)', fontSize: '13px' }}>
        A browser-based IDE for composing VGM chip music using MML syntax.
      </p>
      <div
        style={{
          borderTop: '1px solid var(--border-color)',
          paddingTop: '12px',
          fontSize: '12px',
          color: 'var(--fg-muted, #888)',
          display: 'flex',
          flexDirection: 'column',
          gap: '4px',
        }}
      >
        <span>Powered by React · Monaco Editor · WebAssembly</span>
        <span>Compiler: mml2vgm-rs (Rust → WASM)</span>
      </div>
    </div>
  </Modal>
);

export default AboutDialog;
