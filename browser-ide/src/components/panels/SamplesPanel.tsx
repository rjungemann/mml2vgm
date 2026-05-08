/**
 * SamplesPanel
 *
 * Per-project WAV sample library UI. Samples are stored in IndexedDB keyed by
 * the active document's ID as the project ID. At compile time the compiler
 * worker receives pre-decoded Float32Array data for each referenced sample.
 */

import React, { useCallback, useEffect, useRef, useState } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { useDocumentStore } from '@/stores/documentStore';
import { sampleService } from '@/services/sampleService';
import type { StoredSample } from '@/services/sampleService';

// ============================================================================
// Constants
// ============================================================================

const MAX_PROJECT_PCM_BYTES = 64 * 1024 * 1024;
const WARN_PROJECT_PCM_BYTES = MAX_PROJECT_PCM_BYTES * 0.875; // warn at 87.5%

// ============================================================================
// Helpers
// ============================================================================

const PCM_REF_REGEX = /'@\s+P\s+\d+\s*,\s*"([^"]+)"/g;

function extractReferencedNames(content: string): Set<string> {
    const names = new Set<string>();
    const re = new RegExp(PCM_REF_REGEX.source, 'g');
    let m: RegExpExecArray | null;
    while ((m = re.exec(content)) !== null) {
        names.add(m[1]);
    }
    return names;
}

function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

function formatDate(d: Date): string {
    const date = d instanceof Date ? d : new Date(d);
    return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
}

// ============================================================================
// Types
// ============================================================================

interface PendingDuplicate {
    name: string;
    buf: ArrayBuffer;
}

// ============================================================================
// Component
// ============================================================================

const SamplesPanel: React.FC = () => {
    const { activeDocumentId, activeDocument } = useDocumentStore(
        useShallow((state) => ({
            activeDocumentId: state.activeDocumentId,
            activeDocument: state.activeDocumentId
                ? state.documents.get(state.activeDocumentId) ?? null
                : null,
        }))
    );

    const [samples, setSamples] = useState<Omit<StoredSample, 'pcm'>[]>([]);
    const [error, setError] = useState<string | null>(null);
    const [isDragging, setIsDragging] = useState(false);
    const [renamingName, setRenamingName] = useState<string | null>(null);
    const [renameValue, setRenameValue] = useState('');
    const [confirmDelete, setConfirmDelete] = useState<string | null>(null);

    // Duplicate-resolution queue
    const [duplicateQueue, setDuplicateQueue] = useState<PendingDuplicate[]>([]);
    const [currentDuplicate, setCurrentDuplicate] = useState<PendingDuplicate | null>(null);
    const [duplicateRenaming, setDuplicateRenaming] = useState(false);
    const [duplicateRenameValue, setDuplicateRenameValue] = useState('');

    const fileInputRef = useRef<HTMLInputElement>(null);

    const projectId = activeDocumentId;
    const referencedNames = activeDocument ? extractReferencedNames(activeDocument.content) : new Set<string>();

    // Per-project decoded-PCM usage (rough estimate matching sampleService.put())
    const totalDecodedBytes = samples.reduce((sum, s) => sum + s.size * 4, 0);
    const isNearLimit = totalDecodedBytes >= WARN_PROJECT_PCM_BYTES;

    // Load sample list whenever active document changes
    const refresh = useCallback(async () => {
        if (!projectId) { setSamples([]); return; }
        try {
            const list = await sampleService.list(projectId);
            setSamples(list.sort((a, b) => a.name.localeCompare(b.name)));
        } catch (e) {
            setError((e as Error).message);
        }
    }, [projectId]);

    useEffect(() => { refresh(); }, [refresh]);

    // Dequeue one duplicate for resolution whenever the current one is cleared
    useEffect(() => {
        if (duplicateQueue.length > 0 && !currentDuplicate) {
            const [next, ...rest] = duplicateQueue;
            setCurrentDuplicate(next);
            setDuplicateQueue(rest);
            setDuplicateRenaming(false);
            setDuplicateRenameValue(next.name);
        }
    }, [duplicateQueue, currentDuplicate]);

    // -------------------------------------------------------------------------
    // Upload
    // -------------------------------------------------------------------------

    const uploadFiles = useCallback(async (files: FileList | File[]) => {
        if (!projectId) return;
        setError(null);
        const arr = Array.from(files).filter((f) => f.name.toLowerCase().endsWith('.wav'));
        if (arr.length === 0) {
            setError('Only .wav files are supported.');
            return;
        }

        const toUpload: PendingDuplicate[] = [];
        const toDuplicate: PendingDuplicate[] = [];

        for (const file of arr) {
            const buf = await file.arrayBuffer();
            if (samples.some((s) => s.name === file.name)) {
                toDuplicate.push({ name: file.name, buf });
            } else {
                toUpload.push({ name: file.name, buf });
            }
        }

        // Upload non-duplicates immediately
        for (const { name, buf } of toUpload) {
            try {
                await sampleService.put(projectId, name, buf);
            } catch (e) {
                setError((e as Error).message);
            }
        }
        if (toUpload.length > 0) await refresh();

        // Queue duplicates for user resolution
        if (toDuplicate.length > 0) {
            setDuplicateQueue((prev) => [...prev, ...toDuplicate]);
        }
    }, [projectId, samples, refresh]);

    const handleFileInputChange = useCallback(
        (e: React.ChangeEvent<HTMLInputElement>) => {
            if (e.target.files) uploadFiles(e.target.files);
            e.target.value = '';
        },
        [uploadFiles]
    );

    const handleUploadClick = useCallback(() => {
        fileInputRef.current?.click();
    }, []);

    // -------------------------------------------------------------------------
    // Drag-and-drop
    // -------------------------------------------------------------------------

    const handleDragOver = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        setIsDragging(true);
    }, []);

    const handleDragLeave = useCallback(() => setIsDragging(false), []);

    const handleDrop = useCallback(
        (e: React.DragEvent) => {
            e.preventDefault();
            setIsDragging(false);
            if (e.dataTransfer.files) uploadFiles(e.dataTransfer.files);
        },
        [uploadFiles]
    );

    // -------------------------------------------------------------------------
    // Duplicate resolution
    // -------------------------------------------------------------------------

    const handleDuplicateOverwrite = useCallback(async () => {
        if (!projectId || !currentDuplicate) return;
        try {
            await sampleService.put(projectId, currentDuplicate.name, currentDuplicate.buf);
            await refresh();
        } catch (e) {
            setError((e as Error).message);
        }
        setCurrentDuplicate(null);
    }, [projectId, currentDuplicate, refresh]);

    const handleDuplicateRenameCommit = useCallback(async () => {
        if (!projectId || !currentDuplicate) return;
        const newName = duplicateRenameValue.trim();
        if (!newName) return;
        try {
            await sampleService.put(projectId, newName, currentDuplicate.buf);
            await refresh();
        } catch (e) {
            setError((e as Error).message);
        }
        setCurrentDuplicate(null);
    }, [projectId, currentDuplicate, duplicateRenameValue, refresh]);

    const handleDuplicateRenameKeyDown = useCallback(
        (e: React.KeyboardEvent) => {
            if (e.key === 'Enter') handleDuplicateRenameCommit();
            else if (e.key === 'Escape') { setDuplicateRenaming(false); setDuplicateRenameValue(currentDuplicate?.name ?? ''); }
        },
        [handleDuplicateRenameCommit, currentDuplicate]
    );

    const handleDuplicateSkip = useCallback(() => {
        setCurrentDuplicate(null);
    }, []);

    // -------------------------------------------------------------------------
    // Delete
    // -------------------------------------------------------------------------

    const handleDeleteRequest = useCallback((name: string) => {
        setConfirmDelete(name);
    }, []);

    const handleDeleteConfirm = useCallback(async () => {
        if (!projectId || !confirmDelete) return;
        try {
            await sampleService.delete(projectId, confirmDelete);
            setConfirmDelete(null);
            await refresh();
        } catch (e) {
            setError((e as Error).message);
            setConfirmDelete(null);
        }
    }, [projectId, confirmDelete, refresh]);

    // -------------------------------------------------------------------------
    // Rename
    // -------------------------------------------------------------------------

    const handleRenameStart = useCallback((name: string) => {
        setRenamingName(name);
        setRenameValue(name);
    }, []);

    const handleRenameCommit = useCallback(async () => {
        if (!projectId || !renamingName) return;
        const newName = renameValue.trim();
        if (!newName || newName === renamingName) { setRenamingName(null); return; }
        try {
            await sampleService.rename(projectId, renamingName, newName);
            setRenamingName(null);
            await refresh();
        } catch (e) {
            setError((e as Error).message);
            setRenamingName(null);
        }
    }, [projectId, renamingName, renameValue, refresh]);

    const handleRenameKeyDown = useCallback(
        (e: React.KeyboardEvent) => {
            if (e.key === 'Enter') handleRenameCommit();
            else if (e.key === 'Escape') setRenamingName(null);
        },
        [handleRenameCommit]
    );

    // -------------------------------------------------------------------------
    // Render
    // -------------------------------------------------------------------------

    if (!projectId) {
        return (
            <div style={styles.empty}>
                No document open. Open or create a document to manage its samples.
            </div>
        );
    }

    const missingNames = Array.from(referencedNames).filter(
        (n) => !samples.some((s) => s.name === n)
    );

    return (
        <div
            style={{ ...styles.container, ...(isDragging ? styles.dragging : {}) }}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
        >
            {/* Hidden file input */}
            <input
                ref={fileInputRef}
                type="file"
                accept=".wav"
                multiple
                style={{ display: 'none' }}
                onChange={handleFileInputChange}
            />

            {/* Toolbar */}
            <div style={styles.toolbar}>
                <button style={styles.uploadBtn} onClick={handleUploadClick}>
                    Upload WAV…
                </button>
                <span style={styles.hint}>or drag &amp; drop .wav files here</span>
            </div>

            {/* Error banner */}
            {error && (
                <div style={styles.errorBanner}>
                    {error}
                    <button style={styles.dismissBtn} onClick={() => setError(null)}>✕</button>
                </div>
            )}

            {/* Near-limit warning */}
            {isNearLimit && (
                <div style={styles.limitBanner}>
                    Project storage is near the 64 MB limit ({formatSize(totalDecodedBytes)} decoded PCM used).
                    Delete unused samples to free space.
                </div>
            )}

            {/* Missing-sample warnings */}
            {missingNames.length > 0 && (
                <div style={styles.missingBanner}>
                    Missing samples referenced in this document:&nbsp;
                    {missingNames.map((n, i) => (
                        <span key={n}>
                            <code style={styles.code}>{n}</code>
                            {i < missingNames.length - 1 ? ', ' : ''}
                        </span>
                    ))}
                </div>
            )}

            {/* Delete confirmation */}
            {confirmDelete && (
                <div style={styles.confirmBanner}>
                    Delete <strong>{confirmDelete}</strong>?&nbsp;
                    <button style={styles.dangerBtn} onClick={handleDeleteConfirm}>Delete</button>
                    <button style={styles.cancelBtn} onClick={() => setConfirmDelete(null)}>Cancel</button>
                </div>
            )}

            {/* Duplicate resolution */}
            {currentDuplicate && (
                <div style={styles.duplicateBanner}>
                    {duplicateRenaming ? (
                        <>
                            <span style={styles.duplicateMsg}>Save as:</span>
                            <input
                                autoFocus
                                style={styles.duplicateInput}
                                value={duplicateRenameValue}
                                onChange={(e) => setDuplicateRenameValue(e.target.value)}
                                onKeyDown={handleDuplicateRenameKeyDown}
                                onBlur={handleDuplicateRenameCommit}
                            />
                            <button style={styles.accentBtn} onClick={handleDuplicateRenameCommit}>Save</button>
                            <button style={styles.cancelBtn} onClick={() => setDuplicateRenaming(false)}>Back</button>
                        </>
                    ) : (
                        <>
                            <span style={styles.duplicateMsg}>
                                <strong>{currentDuplicate.name}</strong> already exists.
                            </span>
                            <button style={styles.dangerBtn} onClick={handleDuplicateOverwrite}>Overwrite</button>
                            <button style={styles.accentBtn} onClick={() => { setDuplicateRenaming(true); }}>Rename…</button>
                            <button style={styles.cancelBtn} onClick={handleDuplicateSkip}>Skip</button>
                            {duplicateQueue.length > 0 && (
                                <span style={styles.queueHint}>+{duplicateQueue.length} more</span>
                            )}
                        </>
                    )}
                </div>
            )}

            {/* Sample list */}
            {samples.length === 0 ? (
                <div style={styles.empty}>No samples uploaded yet.</div>
            ) : (
                <div style={styles.list}>
                    {samples.map((s) => {
                        const isReferenced = referencedNames.has(s.name);
                        return (
                            <div key={s.name} style={styles.row}>
                                {/* Name (or inline rename field) */}
                                <div style={styles.nameCell}>
                                    {renamingName === s.name ? (
                                        <input
                                            autoFocus
                                            style={styles.renameInput}
                                            value={renameValue}
                                            onChange={(e) => setRenameValue(e.target.value)}
                                            onBlur={handleRenameCommit}
                                            onKeyDown={handleRenameKeyDown}
                                        />
                                    ) : (
                                        <span
                                            style={styles.sampleName}
                                            title="Double-click to rename"
                                            onDoubleClick={() => handleRenameStart(s.name)}
                                        >
                                            {s.name}
                                        </span>
                                    )}
                                    {isReferenced && (
                                        <span style={styles.badge} title="Referenced in this document">
                                            referenced
                                        </span>
                                    )}
                                </div>

                                {/* Metadata */}
                                <div style={styles.metaCell}>
                                    <span>{formatSize(s.size)}</span>
                                    <span style={styles.sep}>·</span>
                                    <span>{s.sampleRate} Hz</span>
                                    <span style={styles.sep}>·</span>
                                    <span>{s.channels === 1 ? 'mono' : 'stereo'}</span>
                                </div>

                                {/* Date */}
                                <div style={styles.dateCell}>
                                    {formatDate(s.updatedAt)}
                                </div>

                                {/* Actions */}
                                <div style={styles.actionsCell}>
                                    <button
                                        style={styles.iconBtn}
                                        title="Rename"
                                        onClick={() => handleRenameStart(s.name)}
                                    >
                                        ✎
                                    </button>
                                    <button
                                        style={{ ...styles.iconBtn, ...styles.deleteIconBtn }}
                                        title="Delete"
                                        onClick={() => handleDeleteRequest(s.name)}
                                    >
                                        ✕
                                    </button>
                                </div>
                            </div>
                        );
                    })}
                </div>
            )}
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
        fontSize: '12px',
        backgroundColor: 'var(--bg-primary, #1e1e1e)',
        color: 'var(--text-primary, #d4d4d4)',
        transition: 'box-shadow 0.15s',
    },
    dragging: {
        boxShadow: 'inset 0 0 0 2px var(--accent, #007acc)',
    },
    toolbar: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        padding: '6px 8px',
        borderBottom: '1px solid var(--border, #3c3c3c)',
        flexShrink: 0,
    },
    uploadBtn: {
        padding: '3px 10px',
        fontSize: '12px',
        cursor: 'pointer',
        background: 'var(--accent, #007acc)',
        color: '#fff',
        border: 'none',
        borderRadius: '3px',
    },
    hint: {
        color: 'var(--text-muted, #888)',
        fontSize: '11px',
    },
    errorBanner: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '5px 8px',
        background: 'var(--error-bg, #5a1a1a)',
        color: 'var(--error-text, #f48771)',
        borderBottom: '1px solid var(--border, #3c3c3c)',
        flexShrink: 0,
    },
    limitBanner: {
        padding: '5px 8px',
        background: 'var(--error-bg, #5a1a1a)',
        color: 'var(--error-text, #f48771)',
        borderBottom: '1px solid var(--border, #3c3c3c)',
        flexShrink: 0,
        lineHeight: '1.5',
    },
    missingBanner: {
        padding: '5px 8px',
        background: 'var(--warning-bg, #3a2e00)',
        color: 'var(--warning-text, #cca700)',
        borderBottom: '1px solid var(--border, #3c3c3c)',
        flexShrink: 0,
        lineHeight: '1.5',
    },
    confirmBanner: {
        display: 'flex',
        alignItems: 'center',
        gap: '6px',
        padding: '5px 8px',
        background: 'var(--warning-bg, #3a2e00)',
        color: 'var(--text-primary, #d4d4d4)',
        borderBottom: '1px solid var(--border, #3c3c3c)',
        flexShrink: 0,
    },
    duplicateBanner: {
        display: 'flex',
        alignItems: 'center',
        gap: '6px',
        padding: '5px 8px',
        background: 'var(--bg-secondary, #252526)',
        color: 'var(--text-primary, #d4d4d4)',
        borderBottom: '1px solid var(--border, #3c3c3c)',
        flexShrink: 0,
    },
    duplicateMsg: {
        flex: '1 1 0',
        overflow: 'hidden',
        textOverflow: 'ellipsis',
        whiteSpace: 'nowrap',
    },
    duplicateInput: {
        flex: '1 1 0',
        minWidth: 0,
        background: 'var(--input-bg, #3c3c3c)',
        border: '1px solid var(--accent, #007acc)',
        color: 'var(--text-primary, #d4d4d4)',
        padding: '1px 4px',
        fontSize: '12px',
        fontFamily: 'inherit',
        borderRadius: '2px',
    },
    queueHint: {
        flexShrink: 0,
        fontSize: '11px',
        color: 'var(--text-muted, #888)',
    },
    dismissBtn: {
        background: 'none',
        border: 'none',
        color: 'inherit',
        cursor: 'pointer',
        fontSize: '12px',
        padding: '0 4px',
    },
    dangerBtn: {
        flexShrink: 0,
        padding: '2px 8px',
        background: 'var(--error-bg, #5a1a1a)',
        color: 'var(--error-text, #f48771)',
        border: '1px solid var(--error-text, #f48771)',
        borderRadius: '3px',
        cursor: 'pointer',
        fontSize: '12px',
    },
    accentBtn: {
        flexShrink: 0,
        padding: '2px 8px',
        background: 'var(--accent, #007acc)',
        color: '#fff',
        border: 'none',
        borderRadius: '3px',
        cursor: 'pointer',
        fontSize: '12px',
    },
    cancelBtn: {
        flexShrink: 0,
        padding: '2px 8px',
        background: 'none',
        color: 'var(--text-muted, #888)',
        border: '1px solid var(--border, #3c3c3c)',
        borderRadius: '3px',
        cursor: 'pointer',
        fontSize: '12px',
    },
    list: {
        flex: 1,
        overflowY: 'auto',
    },
    row: {
        display: 'flex',
        alignItems: 'center',
        padding: '4px 8px',
        borderBottom: '1px solid var(--border-subtle, #2a2a2a)',
        gap: '8px',
    },
    nameCell: {
        flex: '1 1 0',
        display: 'flex',
        alignItems: 'center',
        gap: '6px',
        minWidth: 0,
        overflow: 'hidden',
    },
    sampleName: {
        overflow: 'hidden',
        textOverflow: 'ellipsis',
        whiteSpace: 'nowrap',
        cursor: 'default',
        userSelect: 'none',
    },
    badge: {
        flexShrink: 0,
        fontSize: '10px',
        padding: '1px 5px',
        background: 'var(--badge-bg, #0e4272)',
        color: 'var(--badge-text, #75beff)',
        borderRadius: '10px',
        fontFamily: 'sans-serif',
    },
    metaCell: {
        flexShrink: 0,
        display: 'flex',
        gap: '2px',
        color: 'var(--text-muted, #888)',
        whiteSpace: 'nowrap',
        fontSize: '11px',
    },
    sep: {
        margin: '0 2px',
        opacity: 0.5,
    },
    dateCell: {
        flexShrink: 0,
        color: 'var(--text-muted, #888)',
        fontSize: '11px',
        whiteSpace: 'nowrap',
    },
    actionsCell: {
        flexShrink: 0,
        display: 'flex',
        gap: '4px',
    },
    iconBtn: {
        background: 'none',
        border: 'none',
        color: 'var(--text-muted, #888)',
        cursor: 'pointer',
        padding: '2px 4px',
        fontSize: '12px',
        borderRadius: '2px',
    },
    deleteIconBtn: {
        color: 'var(--error-text, #f48771)',
    },
    renameInput: {
        flex: 1,
        minWidth: 0,
        background: 'var(--input-bg, #3c3c3c)',
        border: '1px solid var(--accent, #007acc)',
        color: 'var(--text-primary, #d4d4d4)',
        padding: '1px 4px',
        fontSize: '12px',
        fontFamily: 'inherit',
        borderRadius: '2px',
    },
    empty: {
        padding: '16px',
        color: 'var(--text-muted, #888)',
        textAlign: 'center',
        flex: 1,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
    },
    code: {
        fontFamily: 'monospace',
        background: 'var(--code-bg, #2a2a2a)',
        padding: '0 3px',
        borderRadius: '2px',
    },
};

export default SamplesPanel;
