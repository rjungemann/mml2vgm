import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { useDocumentStore } from '@/stores/documentStore';
import {
    parseArpeggios,
    replaceDefinitionBlock,
    serializeArpeggio,
    type ArpeggioDef,
} from '@/utils/instrumentParser';

// ============================================================================
// Note helpers
// ============================================================================

const NOTE_LETTERS = ['c', 'd', 'e', 'f', 'g', 'a', 'b'] as const;
const OCTAVES = [0, 1, 2, 3, 4, 5, 6, 7, 8] as const;

interface ParsedNote {
    letter: string;
    sharp: boolean;
    octave: number;
}

function parseNote(note: string): ParsedNote {
    const m = note.trim().match(/^([a-g])(#|\+)?(\d+)?/i);
    if (!m) return { letter: 'c', sharp: false, octave: 4 };
    return {
        letter: m[1].toLowerCase(),
        sharp: m[2] === '#' || m[2] === '+',
        octave: m[3] !== undefined ? parseInt(m[3], 10) : 4,
    };
}

function formatNote(p: ParsedNote): string {
    return `${p.letter}${p.sharp ? '+' : ''}${p.octave}`;
}

function displayNote(p: ParsedNote): string {
    return `${p.letter.toUpperCase()}${p.sharp ? '♯' : ''}${p.octave}`;
}

function defaultArpeggio(number: number): ArpeggioDef {
    return { number, notes: ['c4', 'e4', 'g4'], startLine: -1, endLine: -1 };
}

// ============================================================================
// Main panel
// ============================================================================

const ArpeggioEditorPanel: React.FC = () => {
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
    const arpeggios = useMemo(() => parseArpeggios(source), [source]);

    const [selectedIdx, setSelectedIdx] = useState(0);
    const [local, setLocal] = useState<ArpeggioDef | null>(null);

    useEffect(() => {
        if (arpeggios.length === 0) { setLocal(null); return; }
        const idx = Math.min(selectedIdx, arpeggios.length - 1);
        setLocal({ ...arpeggios[idx], notes: [...arpeggios[idx].notes] });
    }, [arpeggios, selectedIdx]);

    const applyToDocument = useCallback((updated: ArpeggioDef) => {
        if (!activeDocumentId) return;
        const block = serializeArpeggio(updated);
        const newSource = replaceDefinitionBlock(source, updated.startLine, updated.endLine, block);
        updateDocumentContent(activeDocumentId, newSource);
    }, [activeDocumentId, source, updateDocumentContent]);

    const setLocalAndApply = useCallback((updater: (prev: ArpeggioDef) => ArpeggioDef) => {
        setLocal((prev) => {
            if (!prev) return prev;
            const next = updater(prev);
            applyToDocument(next);
            return next;
        });
    }, [applyToDocument]);

    const handleNoteField = useCallback(
        (i: number, field: keyof ParsedNote, value: string | boolean | number) => {
            setLocalAndApply((prev) => {
                const notes = [...prev.notes];
                const p = parseNote(notes[i] ?? 'c4');
                if (field === 'letter') p.letter = value as string;
                else if (field === 'sharp') p.sharp = value as boolean;
                else if (field === 'octave') p.octave = value as number;
                notes[i] = formatNote(p);
                return { ...prev, notes };
            });
        },
        [setLocalAndApply]
    );

    const handleAddNote = useCallback(() => {
        setLocalAndApply((prev) => {
            const last = prev.notes.length > 0 ? prev.notes[prev.notes.length - 1] : 'c4';
            return { ...prev, notes: [...prev.notes, last] };
        });
    }, [setLocalAndApply]);

    const handleRemoveLast = useCallback(() => {
        setLocalAndApply((prev) => ({ ...prev, notes: prev.notes.slice(0, -1) }));
    }, [setLocalAndApply]);

    const handleAddNew = useCallback(() => {
        if (!activeDocumentId) return;
        const nextNum = arpeggios.length > 0 ? Math.max(...arpeggios.map((a) => a.number)) + 1 : 0;
        const arp = defaultArpeggio(nextNum);
        const newSource = replaceDefinitionBlock(source, -1, -1, serializeArpeggio(arp));
        updateDocumentContent(activeDocumentId, newSource);
        setSelectedIdx(arpeggios.length);
    }, [activeDocumentId, arpeggios, source, updateDocumentContent]);

    const handleDelete = useCallback(() => {
        if (!local || !activeDocumentId) return;
        if (!window.confirm(`Delete Arpeggio ${String(local.number).padStart(3, '0')}?`)) return;
        const lines = source.split('\n');
        updateDocumentContent(
            activeDocumentId,
            [...lines.slice(0, local.startLine), ...lines.slice(local.endLine + 1)].join('\n')
        );
        setSelectedIdx(Math.max(0, selectedIdx - 1));
    }, [local, activeDocumentId, source, updateDocumentContent, selectedIdx]);

    const handleCopyMml = useCallback(() => {
        if (!local) return;
        navigator.clipboard.writeText(serializeArpeggio(local)).catch(console.error);
    }, [local]);

    // ── No document ──────────────────────────────────────────────────────────
    if (!activeDocumentId) {
        return (
            <div style={styles.empty}>
                No document open. Open a GWI file to edit its arpeggios.
            </div>
        );
    }

    // ── No arpeggios ─────────────────────────────────────────────────────────
    if (arpeggios.length === 0) {
        return (
            <div style={styles.container}>
                <div style={styles.empty}>
                    No arpeggio definitions found in this document.
                    <br />
                    <button style={styles.btn} onClick={handleAddNew} className="button primary">
                        + Add Arpeggio
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
                <label style={styles.label}>Arpeggio</label>
                <select
                    style={styles.select}
                    value={selectedIdx}
                    onChange={(e) => setSelectedIdx(Number(e.target.value))}
                >
                    {arpeggios.map((arp, idx) => (
                        <option key={arp.number} value={idx}>
                            {String(arp.number).padStart(3, '0')} ({arp.notes.length} notes)
                        </option>
                    ))}
                </select>
                <button style={styles.btnSmall} onClick={handleAddNew} title="Add new arpeggio">
                    + New
                </button>
                <button
                    style={{ ...styles.btnSmall, color: 'var(--error-text, #f48771)' }}
                    onClick={handleDelete}
                    title="Delete this arpeggio"
                >
                    ✕
                </button>
            </div>

            {/* ── Notes ────────────────────────────────────────────────────── */}
            <div style={styles.body}>
                <div style={{ padding: '8px 10px' }}>
                    <div style={styles.sectionLabel}>Notes ({local.notes.length})</div>
                    <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap', marginBottom: 12 }}>
                        {local.notes.map((note, i) => {
                            const p = parseNote(note);
                            return (
                                <div key={i} style={styles.noteCell}>
                                    <span style={{ fontSize: 10, color: 'var(--fg-muted, #888)' }}>{i}</span>
                                    <select
                                        style={{ ...styles.select, width: 44 }}
                                        value={p.letter}
                                        onChange={(e) => handleNoteField(i, 'letter', e.target.value)}
                                    >
                                        {NOTE_LETTERS.map((l) => (
                                            <option key={l} value={l}>{l.toUpperCase()}</option>
                                        ))}
                                    </select>
                                    <label style={{ fontSize: 10, display: 'flex', alignItems: 'center', gap: 2, color: 'var(--fg-secondary, #aaa)', cursor: 'pointer' }}>
                                        <input
                                            type="checkbox"
                                            checked={p.sharp}
                                            onChange={(e) => handleNoteField(i, 'sharp', e.target.checked)}
                                            style={{ margin: 0 }}
                                        />
                                        ♯
                                    </label>
                                    <select
                                        style={{ ...styles.select, width: 44 }}
                                        value={p.octave}
                                        onChange={(e) => handleNoteField(i, 'octave', parseInt(e.target.value, 10))}
                                    >
                                        {OCTAVES.map((o) => (
                                            <option key={o} value={o}>{o}</option>
                                        ))}
                                    </select>
                                </div>
                            );
                        })}
                    </div>

                    {/* Pattern display */}
                    <div style={styles.sectionLabel}>Pattern</div>
                    <div style={styles.patternDisplay}>
                        {local.notes.length > 0
                            ? local.notes.map((n) => displayNote(parseNote(n))).join('  →  ') + '  → …'
                            : '(empty)'}
                    </div>
                </div>
            </div>

            {/* ── Footer ───────────────────────────────────────────────────── */}
            <div style={styles.footer}>
                <button style={styles.btnSmall} onClick={handleAddNote}>+ Add Note</button>
                <button
                    style={styles.btnSmall}
                    onClick={handleRemoveLast}
                    disabled={local.notes.length === 0}
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
    noteCell: {
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: 3,
        padding: '4px 6px',
        background: 'var(--bg-secondary, #252525)',
        borderRadius: 4,
        border: '1px solid var(--border-color, #333)',
    },
    patternDisplay: {
        padding: '6px 10px',
        background: 'var(--bg-secondary, #252525)',
        borderRadius: 4,
        fontSize: 12,
        color: 'var(--accent, #007acc)',
        letterSpacing: '0.04em',
        wordBreak: 'break-all',
        lineHeight: 1.6,
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

export default ArpeggioEditorPanel;
