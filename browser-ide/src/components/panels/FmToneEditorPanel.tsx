import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { useDocumentStore } from '@/stores/documentStore';
import {
    FM_PARAM_MAX,
    FM_PARAM_NAMES,
    defaultFmInstrument,
    getCarrierOps,
    parseFmInstruments,
    replaceDefinitionBlock,
    serializeFmInstrument,
    type FmInstrumentDef,
    type FmOperator,
    type FmType,
} from '@/utils/instrumentParser';
import { wasmService } from '@/services/wasmService';
import { audioService } from '@/services/audioService';

// ============================================================================
// Algorithm diagram
// ============================================================================

const ALG_DESCRIPTIONS: Record<number, string> = {
    0: 'OP1→OP2→OP3→OP4',
    1: '(OP1+OP2)→OP3→OP4',
    2: 'OP1→OP3, OP2→OP3→OP4',
    3: 'OP1→OP2→OP4, OP3→OP4',
    4: 'OP1→OP2, OP3→OP4 (×2 out)',
    5: 'OP1→(OP2+OP3+OP4) (×3 out)',
    6: 'OP1→OP2, OP3, OP4 (×3 out)',
    7: 'OP1+OP2+OP3+OP4 (×4 out)',
};

interface AlgDiagramProps {
    alg: number;
}

const AlgDiagram: React.FC<AlgDiagramProps> = ({ alg }) => {
    const carriers = getCarrierOps(alg);
    const ops = [0, 1, 2, 3];

    const boxStyle = (opIdx: number): React.CSSProperties => ({
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        width: 36,
        height: 24,
        borderRadius: 3,
        fontSize: 11,
        fontWeight: 'bold',
        border: '1px solid var(--border-color, #444)',
        background: carriers.includes(opIdx)
            ? 'var(--accent, #007acc)'
            : 'var(--bg-tertiary, #2d2d2d)',
        color: carriers.includes(opIdx) ? '#fff' : 'var(--fg-secondary, #aaa)',
    });

    return (
        <div style={{ fontSize: 11 }}>
            <div style={{ display: 'flex', gap: 6, alignItems: 'center', flexWrap: 'wrap' }}>
                {ops.map((op) => (
                    <span key={op} style={boxStyle(op)}>
                        OP{op + 1}
                    </span>
                ))}
            </div>
            <div style={{ marginTop: 4, color: 'var(--fg-muted, #888)', lineHeight: 1.4 }}>
                {ALG_DESCRIPTIONS[alg]}
            </div>
            <div style={{ marginTop: 2, color: 'var(--accent, #007acc)', fontSize: 10 }}>
                ■ = carrier (output)
            </div>
        </div>
    );
};

// ============================================================================
// Number input cell
// ============================================================================

interface ParamInputProps {
    value: number;
    max: number;
    onChange: (v: number) => void;
    isAm?: boolean;
    isCarrier?: boolean;
    highlight?: boolean;
}

const ParamInput: React.FC<ParamInputProps> = ({ value, max, onChange, isAm, isCarrier, highlight }) => {
    const cellStyle: React.CSSProperties = {
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 2,
        padding: '2px 0',
        background: highlight ? 'rgba(0,122,204,0.08)' : undefined,
        borderRadius: 2,
    };

    if (isAm) {
        return (
            <td style={{ textAlign: 'center', padding: '2px 4px' }}>
                <div style={cellStyle}>
                    <input
                        type="checkbox"
                        checked={value === 1}
                        onChange={(e) => onChange(e.target.checked ? 1 : 0)}
                        style={{ margin: 0, cursor: 'pointer' }}
                    />
                </div>
            </td>
        );
    }

    return (
        <td style={{ padding: '2px 4px' }}>
            <div style={cellStyle}>
                <input
                    type="number"
                    min={0}
                    max={max}
                    value={value}
                    onChange={(e) => {
                        const v = Math.max(0, Math.min(max, parseInt(e.target.value, 10) || 0));
                        onChange(v);
                    }}
                    style={{
                        width: 44,
                        padding: '2px 4px',
                        background: 'var(--bg-tertiary, #2d2d2d)',
                        border: `1px solid ${isCarrier ? 'var(--accent, #007acc)' : 'var(--border-color, #444)'}`,
                        borderRadius: 3,
                        color: 'var(--fg-primary, #d4d4d4)',
                        fontSize: 12,
                        textAlign: 'center',
                    }}
                />
            </div>
        </td>
    );
};

// ============================================================================
// Main panel
// ============================================================================

const FmToneEditorPanel: React.FC = () => {
    const { activeDocumentId, activeDocument, updateDocumentContent } = useDocumentStore(
        useShallow((state) => ({
            activeDocumentId: state.activeDocumentId,
            activeDocument: state.activeDocumentId
                ? state.documents.get(state.activeDocumentId) ?? null
                : null,
            updateDocumentContent: state.updateDocumentContent,
        }))
    );

    const source = activeDocument?.content ?? '';

    // Parse all FM instruments from the current document
    const instruments = useMemo(() => parseFmInstruments(source), [source]);

    // Currently selected instrument index (into `instruments`)
    const [selectedIdx, setSelectedIdx] = useState(0);

    // Local editable copy of the selected instrument
    const [local, setLocal] = useState<FmInstrumentDef | null>(null);

    const [isPreviewing, setIsPreviewing] = useState(false);
    const [previewError, setPreviewError] = useState<string | null>(null);

    // Sync local state when selection or parsed instruments change
    useEffect(() => {
        if (instruments.length === 0) {
            setLocal(null);
            return;
        }
        const idx = Math.min(selectedIdx, instruments.length - 1);
        setLocal({ ...instruments[idx], ops: instruments[idx].ops.map((op) => [...op] as FmOperator) as FmInstrumentDef['ops'] });
    }, [instruments, selectedIdx]);

    // Write changes back to document whenever local state changes
    const applyToDocument = useCallback(
        (updated: FmInstrumentDef) => {
            if (!activeDocumentId) return;
            const block = serializeFmInstrument(updated);
            const newSource = replaceDefinitionBlock(source, updated.startLine, updated.endLine, block);
            updateDocumentContent(activeDocumentId, newSource);
        },
        [activeDocumentId, source, updateDocumentContent]
    );

    const setLocalAndApply = useCallback(
        (updater: (prev: FmInstrumentDef) => FmInstrumentDef) => {
            setLocal((prev) => {
                if (!prev) return prev;
                const next = updater(prev);
                applyToDocument(next);
                return next;
            });
        },
        [applyToDocument]
    );

    const handleOpParam = useCallback(
        (opIdx: number, paramIdx: number, value: number) => {
            setLocalAndApply((prev) => {
                const ops = prev.ops.map((op, i) => {
                    if (i !== opIdx) return op;
                    const next = [...op] as FmOperator;
                    next[paramIdx] = value;
                    return next;
                }) as FmInstrumentDef['ops'];
                return { ...prev, ops };
            });
        },
        [setLocalAndApply]
    );

    const handleAlg = useCallback(
        (alg: number) => setLocalAndApply((p) => ({ ...p, alg })),
        [setLocalAndApply]
    );

    const handleFb = useCallback(
        (fb: number) => setLocalAndApply((p) => ({ ...p, fb })),
        [setLocalAndApply]
    );

    const handleType = useCallback(
        (type: FmType) => setLocalAndApply((p) => ({ ...p, type })),
        [setLocalAndApply]
    );

    const handleName = useCallback(
        (name: string) => setLocalAndApply((p) => ({ ...p, name })),
        [setLocalAndApply]
    );

    const handleAddNew = useCallback(() => {
        if (!activeDocumentId) return;
        const nextNum = instruments.length > 0 ? Math.max(...instruments.map((i) => i.number)) + 1 : 0;
        const inst = defaultFmInstrument(nextNum);
        const block = serializeFmInstrument(inst);
        const newSource = replaceDefinitionBlock(source, -1, -1, block);
        updateDocumentContent(activeDocumentId, newSource);
        setSelectedIdx(instruments.length);
    }, [activeDocumentId, instruments, source, updateDocumentContent]);

    const handleDelete = useCallback(() => {
        if (!local || !activeDocumentId) return;
        if (!window.confirm(`Delete FM instrument ${String(local.number).padStart(3, '0')}?`)) return;
        const lines = source.split('\n');
        const before = lines.slice(0, local.startLine);
        const after = lines.slice(local.endLine + 1);
        updateDocumentContent(activeDocumentId, [...before, ...after].join('\n'));
        setSelectedIdx(Math.max(0, selectedIdx - 1));
    }, [local, activeDocumentId, source, updateDocumentContent, selectedIdx]);

    const handleCopyMml = useCallback(() => {
        if (!local) return;
        navigator.clipboard.writeText(serializeFmInstrument(local)).catch(console.error);
    }, [local]);

    const handlePreview = useCallback(async () => {
        if (!local) return;
        setIsPreviewing(true);
        setPreviewError(null);
        try {
            const block = serializeFmInstrument(local);
            const miniMml = `'{

    TitleName   = Preview
    SystemName  = Sega Genesis
    Format      = VGM
    ClockCount  = 192
    Octave-Rev  = FALSE

    PartYM2612  = A

}

${block}

'A1 T120
'A1 @${local.number} v100 l1 o4 c
`;
            const result = await wasmService.compile(miniMml, {
                format: 'vgm',
                target_chips: ['YM2612'] as any[],
                clock_count: 0,
            });
            if (result.data && result.data.length > 0) {
                await audioService.playVGM(result.data, { chips: ['YM2612'] as any[], volume: audioService.getVolume() });
            } else {
                setPreviewError(result.errors?.[0]?.message ?? 'Compile failed');
            }
        } catch (e) {
            setPreviewError((e as Error).message);
        } finally {
            setIsPreviewing(false);
        }
    }, [local]);

    // ── No document ────────────────────────────────────────────────────────────
    if (!activeDocumentId) {
        return (
            <div style={styles.empty}>
                No document open. Open a GWI file to edit its FM instruments.
            </div>
        );
    }

    // ── No instruments yet ─────────────────────────────────────────────────────
    if (instruments.length === 0) {
        return (
            <div style={styles.container}>
                <div style={styles.empty}>
                    No FM instrument definitions found in this document.
                    <br />
                    <button style={styles.btn} onClick={handleAddNew} className="button primary">
                        + Add FM Instrument
                    </button>
                </div>
            </div>
        );
    }

    if (!local) return null;

    const carriers = getCarrierOps(local.alg);

    return (
        <div style={styles.container}>
            {/* ── Toolbar ────────────────────────────────────────────────────── */}
            <div style={styles.toolbar}>
                <label style={styles.label}>Instrument</label>
                <select
                    style={styles.select}
                    value={selectedIdx}
                    onChange={(e) => setSelectedIdx(Number(e.target.value))}
                >
                    {instruments.map((inst, idx) => (
                        <option key={inst.number} value={idx}>
                            {String(inst.number).padStart(3, '0')}{inst.name ? ` "${inst.name}"` : ''}
                        </option>
                    ))}
                </select>

                <label style={styles.label}>Type</label>
                <select
                    style={{ ...styles.select, width: 60 }}
                    value={local.type}
                    onChange={(e) => handleType(e.target.value as FmType)}
                >
                    <option value="M">M</option>
                    <option value="F">F</option>
                </select>

                <label style={styles.label}>Name</label>
                <input
                    type="text"
                    style={{ ...styles.input, flex: 1 }}
                    value={local.name}
                    placeholder="optional"
                    onChange={(e) => handleName(e.target.value)}
                />

                <button style={styles.btnSmall} onClick={handleAddNew} title="Add new FM instrument">
                    + New
                </button>
                <button style={{ ...styles.btnSmall, color: 'var(--error-text, #f48771)' }} onClick={handleDelete} title="Delete this instrument">
                    ✕
                </button>
            </div>

            {/* ── Body ───────────────────────────────────────────────────────── */}
            <div style={styles.body}>
                {/* Left: ALG/FB + diagram */}
                <div style={styles.leftCol}>
                    <div style={styles.sectionLabel}>Algorithm</div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
                        <input
                            type="number"
                            min={0}
                            max={7}
                            value={local.alg}
                            onChange={(e) => handleAlg(Math.max(0, Math.min(7, parseInt(e.target.value, 10) || 0)))}
                            style={{ ...styles.input, width: 48 }}
                        />
                        <input
                            type="range"
                            min={0}
                            max={7}
                            value={local.alg}
                            onChange={(e) => handleAlg(Number(e.target.value))}
                            style={{ flex: 1 }}
                        />
                    </div>

                    <div style={styles.sectionLabel}>Feedback</div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 12 }}>
                        <input
                            type="number"
                            min={0}
                            max={7}
                            value={local.fb}
                            onChange={(e) => handleFb(Math.max(0, Math.min(7, parseInt(e.target.value, 10) || 0)))}
                            style={{ ...styles.input, width: 48 }}
                        />
                        <input
                            type="range"
                            min={0}
                            max={7}
                            value={local.fb}
                            onChange={(e) => handleFb(Number(e.target.value))}
                            style={{ flex: 1 }}
                        />
                    </div>

                    <AlgDiagram alg={local.alg} />
                </div>

                {/* Right: 4-op parameter grid */}
                <div style={styles.rightCol}>
                    <table style={styles.table}>
                        <thead>
                            <tr>
                                <th style={styles.th}>Param</th>
                                {[0, 1, 2, 3].map((op) => (
                                    <th
                                        key={op}
                                        style={{
                                            ...styles.th,
                                            color: carriers.includes(op)
                                                ? 'var(--accent, #007acc)'
                                                : 'var(--fg-muted, #888)',
                                        }}
                                    >
                                        OP{op + 1}
                                        {carriers.includes(op) ? ' ●' : ''}
                                    </th>
                                ))}
                            </tr>
                        </thead>
                        <tbody>
                            {FM_PARAM_NAMES.map((name, paramIdx) => (
                                <tr key={name} style={{ background: paramIdx % 2 === 0 ? 'transparent' : 'rgba(255,255,255,0.02)' }}>
                                    <td style={styles.paramLabel}>{name}</td>
                                    {[0, 1, 2, 3].map((opIdx) => (
                                        <ParamInput
                                            key={opIdx}
                                            value={local.ops[opIdx][paramIdx]}
                                            max={FM_PARAM_MAX[paramIdx]}
                                            isAm={name === 'AM'}
                                            isCarrier={carriers.includes(opIdx) && name === 'TL'}
                                            highlight={name === 'TL' && carriers.includes(opIdx)}
                                            onChange={(v) => handleOpParam(opIdx, paramIdx, v)}
                                        />
                                    ))}
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            </div>

            {/* ── Footer ─────────────────────────────────────────────────────── */}
            <div style={styles.footer}>
                <button
                    className="button primary"
                    style={styles.btn}
                    onClick={handlePreview}
                    disabled={isPreviewing}
                >
                    {isPreviewing ? 'Playing…' : '▶ Preview'}
                </button>
                <button className="button" style={styles.btn} onClick={handleCopyMml}>
                    Copy MML
                </button>
                {previewError && (
                    <span style={{ fontSize: 11, color: 'var(--error-text, #f48771)', flex: 1 }}>
                        {previewError}
                    </span>
                )}
                <span style={{ flex: 1 }} />
                <span style={{ fontSize: 11, color: 'var(--fg-muted, #888)' }}>
                    {instruments.length} instrument{instruments.length !== 1 ? 's' : ''} in document
                    &nbsp;·&nbsp; TL of highlighted OP columns controls output volume (0=loud, 127=silent)
                </span>
            </div>
        </div>
    );
};

// ============================================================================
// Styles
// ============================================================================

const styles: Record<string, React.CSSProperties> = {
    container: {
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        overflow: 'hidden',
        fontFamily: 'var(--font-mono, monospace)',
        fontSize: 12,
        backgroundColor: 'var(--bg-primary, #1e1e1e)',
        color: 'var(--fg-primary, #d4d4d4)',
    },
    empty: {
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 16,
        padding: 24,
        color: 'var(--fg-muted, #888)',
        textAlign: 'center',
        lineHeight: 1.6,
    },
    toolbar: {
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        padding: '6px 10px',
        borderBottom: '1px solid var(--border-color, #333)',
        flexShrink: 0,
        flexWrap: 'wrap',
    },
    label: {
        fontSize: 11,
        color: 'var(--fg-muted, #888)',
        whiteSpace: 'nowrap',
    },
    select: {
        padding: '3px 6px',
        background: 'var(--bg-tertiary, #2d2d2d)',
        border: '1px solid var(--border-color, #444)',
        borderRadius: 4,
        color: 'var(--fg-primary, #d4d4d4)',
        fontSize: 12,
    },
    input: {
        padding: '3px 6px',
        background: 'var(--bg-tertiary, #2d2d2d)',
        border: '1px solid var(--border-color, #444)',
        borderRadius: 4,
        color: 'var(--fg-primary, #d4d4d4)',
        fontSize: 12,
    },
    btn: {
        padding: '3px 10px',
        fontSize: 12,
        cursor: 'pointer',
    },
    btnSmall: {
        padding: '2px 8px',
        fontSize: 11,
        cursor: 'pointer',
        background: 'none',
        border: '1px solid var(--border-color, #444)',
        borderRadius: 4,
        color: 'var(--fg-secondary, #aaa)',
    },
    body: {
        display: 'flex',
        flex: 1,
        overflow: 'hidden',
    },
    leftCol: {
        width: 200,
        flexShrink: 0,
        padding: '10px 12px',
        borderRight: '1px solid var(--border-color, #333)',
        overflowY: 'auto',
    },
    rightCol: {
        flex: 1,
        overflowX: 'auto',
        overflowY: 'auto',
        padding: '6px 10px',
    },
    sectionLabel: {
        fontSize: 11,
        color: 'var(--fg-muted, #888)',
        marginBottom: 4,
        textTransform: 'uppercase',
        letterSpacing: '0.05em',
    },
    table: {
        borderCollapse: 'collapse',
        width: '100%',
        minWidth: 300,
    },
    th: {
        textAlign: 'center',
        padding: '4px 6px',
        fontSize: 11,
        fontWeight: 'bold',
        borderBottom: '1px solid var(--border-color, #333)',
        whiteSpace: 'nowrap',
        color: 'var(--fg-muted, #888)',
    },
    paramLabel: {
        padding: '2px 8px 2px 0',
        fontSize: 11,
        color: 'var(--fg-muted, #888)',
        whiteSpace: 'nowrap',
        fontWeight: 'bold',
    },
    footer: {
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        padding: '6px 10px',
        borderTop: '1px solid var(--border-color, #333)',
        flexShrink: 0,
        flexWrap: 'wrap',
    },
};

export default FmToneEditorPanel;
