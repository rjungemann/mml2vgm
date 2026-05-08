/**
 * Sample Service
 *
 * Project-scoped PCM sample library backed by IndexedDB.
 * Each document has its own sample library keyed by its document ID (projectId).
 * WAV files are decoded client-side via Web Audio API — no WAV decoder in WASM.
 */

import type { StoredSample } from './storageService';

// Re-export for convenience
export type { StoredSample };

// ============================================================================
// Constants
// ============================================================================

const DATABASE_NAME = 'mml2vgm-ide-storage';
const SAMPLES_STORE = 'samples';

const MAX_WAV_BYTES = 4 * 1024 * 1024;        // 4 MB per file
const MAX_PROJECT_PCM_BYTES = 64 * 1024 * 1024; // 64 MB decoded PCM total per project

// ============================================================================
// Helpers
// ============================================================================

function openDb(): Promise<IDBDatabase> {
    return new Promise((resolve, reject) => {
        // Use version 2 — matches storageService.ts DATABASE_VERSION
        const req = indexedDB.open(DATABASE_NAME, 2);
        req.onsuccess = () => resolve(req.result);
        req.onerror = () => reject(new Error(`Failed to open IndexedDB: ${req.error}`));
        // onupgradeneeded is handled by storageService; if it fires here it means
        // storageService hasn't run yet — create the store defensively.
        req.onupgradeneeded = (ev) => {
            const db = (ev.target as IDBOpenDBRequest).result;
            if (!db.objectStoreNames.contains(SAMPLES_STORE)) {
                const store = db.createObjectStore(SAMPLES_STORE, { keyPath: ['projectId', 'name'] });
                store.createIndex('by_project', 'projectId');
            }
        };
    });
}

function audioBufferToFloat32(buf: AudioBuffer): Float32Array {
    const channels = buf.numberOfChannels;
    const length = buf.length;
    if (channels === 1) {
        return buf.getChannelData(0).slice();
    }
    const interleaved = new Float32Array(length * channels);
    for (let i = 0; i < length; i++) {
        for (let c = 0; c < channels; c++) {
            interleaved[i * channels + c] = buf.getChannelData(c)[i];
        }
    }
    return interleaved;
}

// ============================================================================
// SampleService
// ============================================================================

export class SampleService {
    private static instance: SampleService | null = null;

    public static getInstance(): SampleService {
        if (!SampleService.instance) {
            SampleService.instance = new SampleService();
        }
        return SampleService.instance;
    }

    /**
     * Decode a WAV ArrayBuffer and store it under the given project + name.
     * Enforces a 4 MB per-file cap and 64 MB per-project decoded PCM cap.
     */
    async put(projectId: string, name: string, wav: ArrayBuffer): Promise<StoredSample> {
        if (wav.byteLength > MAX_WAV_BYTES) {
            throw new Error(
                `Sample "${name}" is ${(wav.byteLength / 1024 / 1024).toFixed(1)} MB — maximum is 4 MB per file.`
            );
        }

        // Decode via Web Audio API
        const audioCtx = new AudioContext();
        let audioBuf: AudioBuffer;
        try {
            audioBuf = await audioCtx.decodeAudioData(wav.slice(0)); // slice avoids detaching caller's buffer
        } finally {
            audioCtx.close();
        }

        const pcm = audioBufferToFloat32(audioBuf);

        // Per-project size cap
        const existing = await this.list(projectId);
        const currentBytes = existing.reduce((sum, s) => sum + s.size * 4, 0); // rough estimate
        const newBytes = pcm.byteLength;
        if (currentBytes + newBytes > MAX_PROJECT_PCM_BYTES) {
            throw new Error(
                `Adding "${name}" would exceed the 64 MB per-project sample limit.`
            );
        }

        const now = new Date();
        const existingSample = await this.get(projectId, name);
        const sample: StoredSample = {
            projectId,
            name,
            size: wav.byteLength,
            channels: audioBuf.numberOfChannels,
            sampleRate: audioBuf.sampleRate,
            pcm,
            uploadedAt: existingSample?.uploadedAt ?? now,
            updatedAt: now,
        };

        const db = await openDb();
        await new Promise<void>((resolve, reject) => {
            const req = db.transaction(SAMPLES_STORE, 'readwrite')
                .objectStore(SAMPLES_STORE)
                .put(sample);
            req.onsuccess = () => resolve();
            req.onerror = () => reject(new Error(`Failed to store sample "${name}"`));
        });

        return sample;
    }

    /** Retrieve a decoded sample. Returns null if not found. */
    async get(projectId: string, name: string): Promise<StoredSample | null> {
        const db = await openDb();
        return new Promise((resolve, reject) => {
            const req = db.transaction(SAMPLES_STORE)
                .objectStore(SAMPLES_STORE)
                .get([projectId, name]);
            req.onsuccess = () => resolve((req.result as StoredSample) ?? null);
            req.onerror = () => reject(new Error(`Failed to get sample "${name}"`));
        });
    }

    /** List all samples for a project — metadata only (no pcm field). */
    async list(projectId: string): Promise<Omit<StoredSample, 'pcm'>[]> {
        const db = await openDb();
        return new Promise((resolve, reject) => {
            const index = db.transaction(SAMPLES_STORE)
                .objectStore(SAMPLES_STORE)
                .index('by_project');
            const req = index.getAll(IDBKeyRange.only(projectId));
            req.onsuccess = () => {
                const rows = (req.result as StoredSample[]).map(({ pcm: _pcm, ...meta }) => meta);
                resolve(rows);
            };
            req.onerror = () => reject(new Error('Failed to list samples'));
        });
    }

    /** Delete a single sample. */
    async delete(projectId: string, name: string): Promise<void> {
        const db = await openDb();
        return new Promise((resolve, reject) => {
            const req = db.transaction(SAMPLES_STORE, 'readwrite')
                .objectStore(SAMPLES_STORE)
                .delete([projectId, name]);
            req.onsuccess = () => resolve();
            req.onerror = () => reject(new Error(`Failed to delete sample "${name}"`));
        });
    }

    /** Delete all samples for a project (e.g. when a document is closed/deleted). */
    async deleteProject(projectId: string): Promise<void> {
        const db = await openDb();
        return new Promise((resolve, reject) => {
            const tx = db.transaction(SAMPLES_STORE, 'readwrite');
            const store = tx.objectStore(SAMPLES_STORE);
            const index = store.index('by_project');
            const req = index.getAllKeys(IDBKeyRange.only(projectId));
            req.onsuccess = () => {
                const keys = req.result as IDBValidKey[];
                let pending = keys.length;
                if (pending === 0) { resolve(); return; }
                for (const key of keys) {
                    const del = store.delete(key);
                    del.onsuccess = () => { if (--pending === 0) resolve(); };
                    del.onerror = () => reject(new Error('Failed to delete project samples'));
                }
            };
            req.onerror = () => reject(new Error('Failed to enumerate project samples'));
        });
    }

    /** Rename a sample (changes the name key). */
    async rename(projectId: string, oldName: string, newName: string): Promise<void> {
        const sample = await this.get(projectId, oldName);
        if (!sample) throw new Error(`Sample "${oldName}" not found`);
        await this.delete(projectId, oldName);
        const db = await openDb();
        await new Promise<void>((resolve, reject) => {
            const renamed: StoredSample = { ...sample, name: newName, updatedAt: new Date() };
            const req = db.transaction(SAMPLES_STORE, 'readwrite')
                .objectStore(SAMPLES_STORE)
                .put(renamed);
            req.onsuccess = () => resolve();
            req.onerror = () => reject(new Error(`Failed to rename sample to "${newName}"`));
        });
    }

    /**
     * Resolve a set of filenames to their decoded PCM for compilation.
     * Missing names map to null in the returned Map.
     */
    async resolve(
        projectId: string,
        names: string[]
    ): Promise<Map<string, Float32Array | null>> {
        const result = new Map<string, Float32Array | null>();
        await Promise.all(
            names.map(async (name) => {
                const sample = await this.get(projectId, name);
                result.set(name, sample?.pcm ?? null);
            })
        );
        return result;
    }
}

export const sampleService = SampleService.getInstance();
export default SampleService;
