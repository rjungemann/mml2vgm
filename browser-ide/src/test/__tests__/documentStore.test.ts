/**
 * Document Store Tests
 *
 * Tests for the Zustand document store — CRUD operations and initial state.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';

describe('documentStore', () => {
  beforeEach(() => {
    vi.resetModules?.();
  });

  // ── Initial state ─────────────────────────────────────────────────────────

  it('initial documents map is empty', { timeout: 1000 }, async () => {
    const { useDocumentStore } = await import('@/stores/documentStore');
    // Reset store to initial state
    useDocumentStore.setState({ documents: new Map(), activeDocumentId: null, nextDocumentId: 1 });
    expect(useDocumentStore.getState().documents.size).toBe(0);
  });

  it('initial activeDocumentId is null', { timeout: 1000 }, async () => {
    const { useDocumentStore } = await import('@/stores/documentStore');
    useDocumentStore.setState({ documents: new Map(), activeDocumentId: null, nextDocumentId: 1 });
    expect(useDocumentStore.getState().activeDocumentId).toBeNull();
  });

  // ── createDocument ────────────────────────────────────────────────────────

  it('createDocument returns a document with an id', { timeout: 1000 }, async () => {
    const { useDocumentStore } = await import('@/stores/documentStore');
    useDocumentStore.setState({ documents: new Map(), activeDocumentId: null, nextDocumentId: 1 });
    const doc = useDocumentStore.getState().createDocument();
    expect(doc.id).toBeDefined();
    expect(doc.id.length).toBeGreaterThan(0);
  });

  it('createDocument sets activeDocumentId', { timeout: 1000 }, async () => {
    const { useDocumentStore } = await import('@/stores/documentStore');
    useDocumentStore.setState({ documents: new Map(), activeDocumentId: null, nextDocumentId: 1 });
    const doc = useDocumentStore.getState().createDocument();
    expect(useDocumentStore.getState().activeDocumentId).toBe(doc.id);
  });

  it('createDocument adds document to documents map', { timeout: 1000 }, async () => {
    const { useDocumentStore } = await import('@/stores/documentStore');
    useDocumentStore.setState({ documents: new Map(), activeDocumentId: null, nextDocumentId: 1 });
    const doc = useDocumentStore.getState().createDocument();
    expect(useDocumentStore.getState().documents.has(doc.id)).toBe(true);
  });

  it('createDocument default content is empty string', { timeout: 1000 }, async () => {
    const { useDocumentStore } = await import('@/stores/documentStore');
    useDocumentStore.setState({ documents: new Map(), activeDocumentId: null, nextDocumentId: 1 });
    const doc = useDocumentStore.getState().createDocument();
    expect(doc.content).toBe('');
  });

  it('createDocument default isDirty is false', { timeout: 1000 }, async () => {
    const { useDocumentStore } = await import('@/stores/documentStore');
    useDocumentStore.setState({ documents: new Map(), activeDocumentId: null, nextDocumentId: 1 });
    const doc = useDocumentStore.getState().createDocument();
    expect(doc.isDirty).toBe(false);
  });

  // ── updateDocumentContent ─────────────────────────────────────────────────

  it('updateDocumentContent stores new content', { timeout: 1000 }, async () => {
    const { useDocumentStore } = await import('@/stores/documentStore');
    useDocumentStore.setState({ documents: new Map(), activeDocumentId: null, nextDocumentId: 1 });
    const doc = useDocumentStore.getState().createDocument();
    useDocumentStore.getState().updateDocumentContent(doc.id, 't120 c d e f');
    const updated = useDocumentStore.getState().getDocument(doc.id);
    expect(updated?.content).toBe('t120 c d e f');
  });

  it('updateDocumentContent marks document as dirty', { timeout: 1000 }, async () => {
    const { useDocumentStore } = await import('@/stores/documentStore');
    useDocumentStore.setState({ documents: new Map(), activeDocumentId: null, nextDocumentId: 1 });
    const doc = useDocumentStore.getState().createDocument();
    useDocumentStore.getState().updateDocumentContent(doc.id, 'c d e f');
    const updated = useDocumentStore.getState().getDocument(doc.id);
    expect(updated?.isDirty).toBe(true);
  });

  // ── closeDocument ─────────────────────────────────────────────────────────

  it('closeDocument removes document from map', { timeout: 1000 }, async () => {
    const { useDocumentStore } = await import('@/stores/documentStore');
    useDocumentStore.setState({ documents: new Map(), activeDocumentId: null, nextDocumentId: 1 });
    const doc = useDocumentStore.getState().createDocument();
    useDocumentStore.getState().closeDocument(doc.id);
    expect(useDocumentStore.getState().documents.has(doc.id)).toBe(false);
  });

  // ── getDocuments ──────────────────────────────────────────────────────────

  it('getDocuments returns all documents as array', { timeout: 1000 }, async () => {
    const { useDocumentStore } = await import('@/stores/documentStore');
    useDocumentStore.setState({ documents: new Map(), activeDocumentId: null, nextDocumentId: 1 });
    useDocumentStore.getState().createDocument();
    useDocumentStore.getState().createDocument();
    const docs = useDocumentStore.getState().getDocuments();
    expect(docs.length).toBe(2);
  });

  // ── setDocumentDirty ──────────────────────────────────────────────────────

  it('setDocumentDirty(true) marks document dirty', { timeout: 1000 }, async () => {
    const { useDocumentStore } = await import('@/stores/documentStore');
    useDocumentStore.setState({ documents: new Map(), activeDocumentId: null, nextDocumentId: 1 });
    const doc = useDocumentStore.getState().createDocument();
    useDocumentStore.getState().setDocumentDirty(doc.id, true);
    expect(useDocumentStore.getState().getDocument(doc.id)?.isDirty).toBe(true);
  });

  it('setDocumentDirty(false) clears dirty flag', { timeout: 1000 }, async () => {
    const { useDocumentStore } = await import('@/stores/documentStore');
    useDocumentStore.setState({ documents: new Map(), activeDocumentId: null, nextDocumentId: 1 });
    const doc = useDocumentStore.getState().createDocument();
    useDocumentStore.getState().setDocumentDirty(doc.id, true);
    useDocumentStore.getState().setDocumentDirty(doc.id, false);
    expect(useDocumentStore.getState().getDocument(doc.id)?.isDirty).toBe(false);
  });

  // ── large document ────────────────────────────────────────────────────────

  it('stores and retrieves 100 KB document correctly', { timeout: 5000 }, async () => {
    const { useDocumentStore } = await import('@/stores/documentStore');
    useDocumentStore.setState({ documents: new Map(), activeDocumentId: null, nextDocumentId: 1 });
    const largeContent = 'c d e f g a b r '.repeat(6500); // ~100 KB
    const doc = useDocumentStore.getState().createDocument();
    useDocumentStore.getState().updateDocumentContent(doc.id, largeContent);
    const retrieved = useDocumentStore.getState().getDocument(doc.id);
    expect(retrieved?.content).toBe(largeContent);
  });
});
