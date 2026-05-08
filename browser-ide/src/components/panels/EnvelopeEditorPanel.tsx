import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { useDocumentStore } from '@/stores/documentStore';
import {
    parseEnvelopes,
    replaceDefinitionBlock,
    serializeEnvelope,
    type EnvelopeDef,
} from '@/utils/instrumentParser';

// ============================================================================
// Bar chart preview
// ============================================================================

const BarChart: React.FC<{ steps: number[] }> = ({ steps }) => {
    const chartHeight = 60;
    const barWidth = Math.max(4, Math.min(16, Math.floor(220 / Math.max(steps.length, 1))));

    return (
        <div
            style={{
                display: 'flex',
                alignItems: 'flex-end',
                height: chartHeight,
                gap: 1,
                padding: '4px 8px',
                background: 'var(--bg-secondary, #252525)',
                borderRadius: 4,
                overflowX: 'auto',
                minHeight: chartHeight,
            }}
        >
            {steps.map((v, i) => (
                <div
                    key={i}
                    title={`Step ${i}: ${v}`}
                    style={{
                        width: barWidth,
                        flexShrink: 0,
                        height: `${Math.max(1, Math.round((v / 127) * (chartHeight - 8)))}px`,
                        background: 'var(--accent, #007acc)',
                        borderRadius: '2px 2px 0 0',
                    }}
                />
            ))}
            {steps.length === 0 && (
                <span style={{ color: 'var(--fg-muted, #888)', fontSize: 11, alignSelf: 'center' }}>
                    No steps
                </span>
            )}
        </div>
    );
};

// ============================================================================
// Helpers
// ============================================================================

function defaultEnvelope(number: number): EnvelopeDef {
    return { number, steps: [0, 16, 48, 96, 127, 96, 64, 32, 16, 8, 4, 0], startLine: -1, endLine: -1 };
}

// ============================================================================
// Main panel
// ============================================================================

const EnvelopeEditorPanel: React.FC = () => {
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
    const envelopes = useMemo(() => parseEnvelopes(source), [source]);

    const [selectedIdx, setSelectedIdx] = useState(0);
    const [local, setLocal] = useState<EnvelopeDef | null>(null);

    useEffect(() => {
        if (envelopes.length === 0) { setLocal(null); return; }
        const idx = Math.min(selectedIdx, envelopes.length - 1);
        setLocal({ ...envelopes[idx], steps: [...envelopes[idx].steps] });
    }, [envelopes, selectedIdx]);

    const applyToDocument = useCallback((updated: EnvelopeDef) => {
        if (!activeDocumentId) return;
        const block = serializeEnvelope(updated);
        const newSource = replaceDefinitionBlock(source, updated.startLine, updated.endLine, block);
        updateDocumentContent(activeDocumentId, newSource);
    }, [activeDocumentId, source, updateDocumentContent]);

    const setLocalAndApply = useCallback((updater: (prev: EnvelopeDef) => EnvelopeDef) => {
        setLocal((prev) => {
            if (!prev) return prev;
            const next = updater(prev);
            applyToDocument(next);
            return next;
        });
    }, [applyToDocument]);

    const handleStepChange = useCallback((i: number, raw: string) => {
        const v = Math.max(0, Math.min(127, parseInt(raw, 10) || 0));
        setLocalAndApply((prev) => {
            const steps = [...prev.steps];
            steps[i] = v;
            return { ...prev, steps };
        });
    }, [setLocalAndApply]);

    const handleAddStep = useCallback(() => {
        setLocalAndApply((prev) => ({ ...prev, steps: [...prev.steps, 0] }));
    }, [setLocalAndApply]);

    const handleRemoveLast = useCallback(() => {
        setLocalAndApply((prev) => ({ ...prev, steps: prev.steps.slice(0, -1) }));
    }, [setLocalAndApply]);

    const handleAddNew = useCallback(() => {
        if (!activeDocumentId) return;
        const nextNum = envelopes.length > 0 ? Math.max(...envelopes.map((e) => e.number)) + 1 : 0;
        const env = defaultEnvelope(nextNum);
        const newSource = replaceDefinitionBlock(source, -1, -1, serializeEnvelope(env));
        updateDocumentContent(activeDocumentId, newSource);
        setSelectedIdx(envelopes.length);
    }, [activeDocumentId, envelopes, source, updateDocumentContent]);

    const handleDelete = useCallback(() => {
        if (!local || !activeDocumentId) return;
        if (!window.confirm(`Delete Envelope ${String(local.number).padStart(3, '0')}?`)) return;
        const lines = source.split('\n');
        updateDocumentContent(
            activeDocumentId,
            [...lines.slice(0, local.startLine), ...lines.slice(local.endLine + 1)].join('\n')
        );
        setSelectedIdx(Math.max(0, selectedIdx - 1));
    }, [local, activeDocumentId, source, updateDocumentContent, selectedIdx]);

    const handleCopyMml = useCallback(() => {
        if (!local) return;
        navigator.clipboard.writeText(serializeEnvelope(local)).catch(console.error);
    }, [local]);

    // ── No document ──────────────────────────────────────────────────────────
    if (!activeDocumentId) {
        return (
            <div style={styles.empty}>
                No document open. Open a GWI file to edit its envelopes.
            </div>
        );
    }

    // ── No envelopes ─────────────────────────────────────────────────────────
    if (envelopes.length === 0) {
        return (
            <div style={styles.container}>
                <div style={styles.empty}>
                    No envelope definitions found in this document.
                    <br />
                    <button style={styles.btn} onClick={handleAddNew} className="button primary">
                        + Add Envelope
                    </button>
                </div>
            </div>
        );
    }

    if (!local) return null;

    return (
        <div style={styles.container}>
            {/* ── Toolbar ──────────────────────────────────────────────────── */}
            <div style={styles.toolbar}>
                <label style={styles.label}>Envelope</label>
                <select
                    style={styles.select}
                    value={selectedIdx}
                    onChange={(e) => setSelectedIdx(Number(e.target.value))}
                >
                    {envelopes.map((env, idx) => (
                        <option key={env.number} value={idx}>
                            {String(env.number).padStart(3, '0')} ({env.steps.length} steps)
                        </option>
                    ))}
                </select>
                <button style={styles.btnSmall} onClick={handleAddNew} title="Add new envelope">
                    + New
                </button>
                <button
                    style={{ ...styles.btnSmall, color: 'var(--error-text, #f48771)' }}
                    onClick={handleDelete}
                    title="Delete this envelope"
                >
                    ✕
                </button>
            </div>

            {/* ── Steps ────────────────────────────────────────────────────── */}
            <div style={styles.body}>
                <div style={{ padding: '8px 10px' }}>
                    <div style={styles.sectionLabel}>Steps ({local.steps.length})</div>
                    <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap', marginBottom: 12 }}>
                        {local.steps.map((v, i) => (
                            <div key={i} style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 2 }}>
                                <span style={{ fontSize: 10, color: 'var(--fg-muted, #888)' }}>{i}</span>
                                <input
                                    type="number"
                                    min={0}
                                    max={127}
                                    value={v}
                                    onChange={(e) => handleStepChange(i, e.target.value)}
                                    style={styles.stepInput}
                                />
                            </div>
                        ))}
                    </div>

                    <div style={styles.sectionLabel}>Volume Curve</div>
                    <BarChart steps={local.steps} />
                </div>
            </div>

            {/* ── Footer ───────────────────────────────────────────────────── */}
            <div style={styles.footer}>
                <button style={styles.btnSmall} onClick={handleAddStep}>+ Add Step</button>
                <button
                    style={styles.btnSmall}
                    onClick={handleRemoveLast}
                    disabled={local.steps.length === 0}
                >
                    − Remove Last
                </button>
                <span style={{ flex: 1 }} />
                <button className="button" style={styles.btn} onClick={handleCopyMml}>
                    Copy MML
                </button>
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
        flex: 1,
        overflowY: 'auto',
    },
    sectionLabel: {
        fontSize: 11,
        color: 'var(--fg-muted, #888)',
        marginBottom: 4,
        textTransform: 'uppercase',
        letterSpacing: '0.05em',
    },
    stepInput: {
        width: 46,
        padding: '2px 4px',
        background: 'var(--bg-tertiary, #2d2d2d)',
        border: '1px solid var(--border-color, #444)',
        borderRadius: 3,
        color: 'var(--fg-primary, #d4d4d4)',
        fontSize: 12,
        textAlign: 'center',
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

export default EnvelopeEditorPanel;
