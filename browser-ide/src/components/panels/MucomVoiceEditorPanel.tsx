/**
 * Mucom88 Voice Editor Panel
 *
 * Provides a form-based editor for the MUCOM88 FM voice (#VOICE) format.
 * Supports editing all OPN FM operator parameters (ALG, FB, AR, DR, SR, RR,
 * SL, TL, KS, ML, DT) for all four operators, and generates / parses
 * #VOICE blocks in the document.
 */

import React, { useState, useCallback } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { useDocumentStore } from '@/stores/documentStore';

// ─── Types ────────────────────────────────────────────────────────────────────

interface OperatorParams {
  AR: number;  // Attack Rate   0-31
  DR: number;  // Decay Rate    0-31
  SR: number;  // Sustain Rate  0-31
  RR: number;  // Release Rate  0-15
  SL: number;  // Sustain Level 0-15
  TL: number;  // Total Level   0-127
  KS: number;  // Key Scale     0-3
  ML: number;  // Multiple      0-15
  DT: number;  // Detune        0-7
}

interface VoiceParams {
  number: number;
  name: string;
  ALG: number;  // Algorithm 0-7
  FB:  number;  // Feedback  0-7
  ops: [OperatorParams, OperatorParams, OperatorParams, OperatorParams];
}

const DEFAULT_OP: OperatorParams = { AR: 31, DR: 0, SR: 0, RR: 7, SL: 0, TL: 0, KS: 0, ML: 1, DT: 0 };

function defaultVoice(number = 0): VoiceParams {
  return {
    number,
    name: `Voice ${number}`,
    ALG: 0,
    FB: 0,
    ops: [{ ...DEFAULT_OP }, { ...DEFAULT_OP }, { ...DEFAULT_OP }, { ...DEFAULT_OP }],
  };
}

// ─── Parsing ─────────────────────────────────────────────────────────────────

const VOICE_BLOCK_RE = /#VOICE\s*(\d+)?\s*\{([\s\S]*?)\}/g;

function parseVoiceBlock(block: string, index: number): VoiceParams {
  const voice = defaultVoice(index);
  const kv = (key: string): number | null => {
    const m = block.match(new RegExp(`\\b${key}\\s*=\\s*(-?\\d+)`));
    return m ? parseInt(m[1], 10) : null;
  };
  const pick = (v: number | null, def: number) => (v !== null ? v : def);

  voice.ALG = pick(kv('ALG'), 0);
  voice.FB  = pick(kv('FB'),  0);

  // Operators — look for op1/op2/op3/op4 sub-blocks or flat OP1_AR etc.
  for (let op = 0; op < 4; op++) {
    const prefix = `OP${op + 1}_`;
    voice.ops[op] = {
      AR: pick(kv(`${prefix}AR`), DEFAULT_OP.AR),
      DR: pick(kv(`${prefix}DR`), DEFAULT_OP.DR),
      SR: pick(kv(`${prefix}SR`), DEFAULT_OP.SR),
      RR: pick(kv(`${prefix}RR`), DEFAULT_OP.RR),
      SL: pick(kv(`${prefix}SL`), DEFAULT_OP.SL),
      TL: pick(kv(`${prefix}TL`), DEFAULT_OP.TL),
      KS: pick(kv(`${prefix}KS`), DEFAULT_OP.KS),
      ML: pick(kv(`${prefix}ML`), DEFAULT_OP.ML),
      DT: pick(kv(`${prefix}DT`), DEFAULT_OP.DT),
    };
  }
  return voice;
}

function parseVoicesFromContent(content: string): VoiceParams[] {
  const voices: VoiceParams[] = [];
  let m: RegExpExecArray | null;
  VOICE_BLOCK_RE.lastIndex = 0;
  while ((m = VOICE_BLOCK_RE.exec(content)) !== null) {
    const num = m[1] ? parseInt(m[1], 10) : voices.length;
    voices.push(parseVoiceBlock(m[2], num));
  }
  return voices.length ? voices : [defaultVoice(0)];
}

// ─── Serialisation ───────────────────────────────────────────────────────────

function serializeVoice(v: VoiceParams): string {
  const lines = [`#VOICE ${v.number} {`, `  ALG=${v.ALG} FB=${v.FB}`];
  v.ops.forEach((op, i) => {
    const p = `  OP${i + 1}`;
    lines.push(
      `${p}  AR=${op.AR} DR=${op.DR} SR=${op.SR} RR=${op.RR} SL=${op.SL} TL=${op.TL} KS=${op.KS} ML=${op.ML} DT=${op.DT}`
    );
  });
  lines.push('}');
  return lines.join('\n');
}

function spliceVoiceBlock(content: string, voice: VoiceParams): string {
  // Replace matching #VOICE N { ... } or append
  const re = new RegExp(`#VOICE\\s*${voice.number}\\s*\\{[\\s\\S]*?\\}`, 'g');
  const newBlock = serializeVoice(voice);
  if (re.test(content)) {
    re.lastIndex = 0;
    return content.replace(re, newBlock);
  }
  return content.trimEnd() + '\n\n' + newBlock + '\n';
}

// ─── Sub-components ───────────────────────────────────────────────────────────

const FIELD_RANGES: Record<keyof OperatorParams, [number, number]> = {
  AR: [0, 31], DR: [0, 31], SR: [0, 31], RR: [0, 15],
  SL: [0, 15], TL: [0, 127], KS: [0, 3], ML: [0, 15], DT: [0, 7],
};

interface OperatorEditorProps {
  label: string;
  op: OperatorParams;
  onChange: (op: OperatorParams) => void;
}

const OperatorEditor: React.FC<OperatorEditorProps> = ({ label, op, onChange }) => {
  const set = (key: keyof OperatorParams, raw: string) => {
    const [min, max] = FIELD_RANGES[key];
    let v = parseInt(raw, 10);
    if (isNaN(v)) v = 0;
    v = Math.max(min, Math.min(max, v));
    onChange({ ...op, [key]: v });
  };

  return (
    <div style={{ marginBottom: 6 }}>
      <div style={{ fontWeight: 600, marginBottom: 2, color: 'var(--accent, #569cd6)' }}>{label}</div>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
        {(Object.keys(FIELD_RANGES) as (keyof OperatorParams)[]).map(key => (
          <label key={key} style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', fontSize: 10 }}>
            <span style={{ color: 'var(--text-muted, #888)', marginBottom: 1 }}>{key}</span>
            <input
              type="number"
              min={FIELD_RANGES[key][0]}
              max={FIELD_RANGES[key][1]}
              value={op[key]}
              onChange={e => set(key, e.target.value)}
              style={{ width: 38, fontSize: 11, textAlign: 'center', background: 'var(--bg-secondary, #252526)', color: 'var(--text, #ccc)', border: '1px solid var(--border, #444)', borderRadius: 2, padding: '1px 2px' }}
            />
          </label>
        ))}
      </div>
    </div>
  );
};

// ─── Main Panel ───────────────────────────────────────────────────────────────

const MucomVoiceEditorPanel: React.FC = () => {
  const { activeDocumentId, documents, updateDocumentContent } = useDocumentStore(
    useShallow((state) => ({
      activeDocumentId: state.activeDocumentId,
      documents: state.documents,
      updateDocumentContent: state.updateDocumentContent,
    }))
  );

  const activeDocument = activeDocumentId ? documents.get(activeDocumentId) : null;

  const [voices, setVoices] = useState<VoiceParams[]>(() =>
    activeDocument ? parseVoicesFromContent(activeDocument.content) : [defaultVoice(0)]
  );
  const [selectedIdx, setSelectedIdx] = useState(0);

  const voice = voices[selectedIdx] ?? defaultVoice(0);

  const updateVoice = useCallback((updated: VoiceParams) => {
    setVoices(prev => prev.map((v, i) => (i === selectedIdx ? updated : v)));
  }, [selectedIdx]);

  const handleParse = useCallback(() => {
    if (!activeDocument) return;
    const parsed = parseVoicesFromContent(activeDocument.content);
    setVoices(parsed.length ? parsed : [defaultVoice(0)]);
    setSelectedIdx(0);
  }, [activeDocument]);

  const handleApply = useCallback(() => {
    if (!activeDocumentId || !activeDocument) return;
    const updated = spliceVoiceBlock(activeDocument.content, voice);
    updateDocumentContent(activeDocumentId, updated);
  }, [activeDocumentId, activeDocument, voice, updateDocumentContent]);

  const handleAddVoice = useCallback(() => {
    const next = defaultVoice(voices.length);
    setVoices(prev => [...prev, next]);
    setSelectedIdx(voices.length);
  }, [voices.length]);

  return (
    <div style={{ padding: '6px', fontSize: '12px', overflowY: 'auto', height: '100%', boxSizing: 'border-box' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6, flexWrap: 'wrap' }}>
        <h3 style={{ margin: 0, fontSize: '13px' }}>MUCOM88 Voice Editor</h3>
        <button className="button small" onClick={handleParse}>Parse</button>
        <button className="button small" onClick={handleApply}>Apply</button>
        <button className="button small secondary" onClick={handleAddVoice}>+ Voice</button>
      </div>

      {/* Voice selector */}
      <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap', marginBottom: 8 }}>
        {voices.map((v, i) => (
          <button
            key={i}
            className={`button small${i === selectedIdx ? '' : ' secondary'}`}
            onClick={() => setSelectedIdx(i)}
            style={{ minWidth: 32 }}
          >
            {v.number}
          </button>
        ))}
      </div>

      {/* Voice global params */}
      <div style={{ display: 'flex', gap: 16, marginBottom: 8 }}>
        <label style={{ fontSize: 11 }}>
          <span style={{ color: 'var(--text-muted, #888)', marginRight: 4 }}>Voice #</span>
          <input
            type="number" min={0} max={255} value={voice.number}
            onChange={e => updateVoice({ ...voice, number: parseInt(e.target.value, 10) || 0 })}
            style={{ width: 48, fontSize: 11, background: 'var(--bg-secondary)', color: 'var(--text)', border: '1px solid var(--border)', borderRadius: 2, padding: '1px 2px' }}
          />
        </label>
        <label style={{ fontSize: 11 }}>
          <span style={{ color: 'var(--text-muted, #888)', marginRight: 4 }}>ALG</span>
          <input
            type="number" min={0} max={7} value={voice.ALG}
            onChange={e => updateVoice({ ...voice, ALG: Math.max(0, Math.min(7, parseInt(e.target.value, 10) || 0)) })}
            style={{ width: 38, fontSize: 11, background: 'var(--bg-secondary)', color: 'var(--text)', border: '1px solid var(--border)', borderRadius: 2, padding: '1px 2px' }}
          />
        </label>
        <label style={{ fontSize: 11 }}>
          <span style={{ color: 'var(--text-muted, #888)', marginRight: 4 }}>FB</span>
          <input
            type="number" min={0} max={7} value={voice.FB}
            onChange={e => updateVoice({ ...voice, FB: Math.max(0, Math.min(7, parseInt(e.target.value, 10) || 0)) })}
            style={{ width: 38, fontSize: 11, background: 'var(--bg-secondary)', color: 'var(--text)', border: '1px solid var(--border)', borderRadius: 2, padding: '1px 2px' }}
          />
        </label>
        <label style={{ fontSize: 11 }}>
          <span style={{ color: 'var(--text-muted, #888)', marginRight: 4 }}>Name</span>
          <input
            type="text" value={voice.name}
            onChange={e => updateVoice({ ...voice, name: e.target.value })}
            style={{ width: 80, fontSize: 11, background: 'var(--bg-secondary)', color: 'var(--text)', border: '1px solid var(--border)', borderRadius: 2, padding: '1px 4px' }}
          />
        </label>
      </div>

      {/* Operator editors */}
      {voice.ops.map((op, i) => (
        <OperatorEditor
          key={i}
          label={`OP${i + 1}`}
          op={op}
          onChange={updated => {
            const newOps = voice.ops.map((o, j) => (j === i ? updated : o)) as VoiceParams['ops'];
            updateVoice({ ...voice, ops: newOps });
          }}
        />
      ))}

      <div style={{ marginTop: 6, color: 'var(--text-muted, #666)', fontSize: 11, lineHeight: 1.4 }}>
        Edit FM voice parameters above. Press <strong>Apply</strong> to write the
        selected voice as a <code>#VOICE</code> block in the document.
      </div>
    </div>
  );
};

export default MucomVoiceEditorPanel;
