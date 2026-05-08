/**
 * Document Store
 * 
 * Manages all documents in the editor, including content, state, and history.
 */

import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';
import type { Document, MMLLanguage, CompileError } from '@/types';
import { formatService, getFormatFromExtension } from '@/services/formatService';
import { sampleService } from '@/services/sampleService';

// ============================================================================
// Types
// ============================================================================

interface DocumentState {
    // All open documents
    documents: Map<string, Document>;
    
    // Currently active document ID
    activeDocumentId: string | null;
    
    // Next document ID counter
    nextDocumentId: number;
}

interface DocumentActions {
    // Create a new document
    createDocument: (language?: MMLLanguage) => Document;
    
    // Open a document from file
    openDocument: (file: File) => Promise<Document>;
    
    // Close a document
    closeDocument: (id: string) => void;
    
    // Close all documents
    closeAllDocuments: () => void;
    
    // Set active document
    setActiveDocument: (id: string) => void;
    
    // Update document content
    updateDocumentContent: (id: string, content: string) => void;
    
    // Update document filename
    updateDocumentFilename: (id: string, filename: string) => void;
    
    // Update document file handle
    updateDocumentFileHandle: (id: string, handle: FileSystemFileHandle | undefined) => void;
    
    // Update document language
    updateDocumentLanguage: (id: string, language: MMLLanguage) => void;
    
    // Set document as dirty/clean
    setDocumentDirty: (id: string, isDirty: boolean) => void;
    
    // Set compilation results
    setCompileResults: (
        id: string,
        success: boolean,
        errors: CompileError[],
        data?: Uint8Array
    ) => void;
    
    // Get active document
    getActiveDocument: () => Document | null;
    
    // Get document by ID
    getDocument: (id: string) => Document | undefined;
    
    // Check if any document is dirty
    hasDirtyDocuments: () => boolean;
    
    // Get all documents as array
    getDocuments: () => Document[];
}

// ============================================================================
// Store Definition
// ============================================================================

type DocumentStore = DocumentState & DocumentActions;

const initialState: DocumentState = {
    documents: new Map(),
    activeDocumentId: null,
    nextDocumentId: 1,
};

export const useDocumentStore = create<DocumentStore>()(
    persist(
        (set, get) => ({
            ...initialState,
            
            // ============================================================
            // Actions
            // ============================================================
            
            createDocument: (language: MMLLanguage = 'gwi') => {
                const state = get();
                const id = `doc-${state.nextDocumentId++}`;
                const doc: Document = {
                    id,
                    filename: `Untitled-${state.nextDocumentId - 1}.gwi`,
                    content: '',
                    language,
                    encoding: 'UTF-8',
                    isDirty: false,
                    lastCompileTime: null,
                    lastCompileSuccess: false,
                    lastCompileErrors: [],
                };
                
                set({
                    documents: new Map(state.documents).set(id, doc),
                    activeDocumentId: id,
                });
                
                return doc;
            },
            
            openDocument: async (file: File) => {
                const state = get();
                const content = await file.text();
                const id = `doc-${state.nextDocumentId++}`;
                
                // Use formatService for format detection
                // Try extension first, then content-based detection
                const detectedFormat = formatService.detectFormat(content, file.name);
                const language: MMLLanguage = detectedFormat.format;
                
                // Fallback to extension-based detection if formatService returns default
                const fallbackLanguage = getLanguageFromExtension(file.name);
                const finalLanguage = language !== 'gwi' || detectedFormat.confidence > 0 
                    ? language 
                    : fallbackLanguage;
                
                const doc: Document = {
                    id,
                    filename: file.name,
                    content,
                    language: finalLanguage,
                    encoding: 'UTF-8',
                    isDirty: false,
                    lastCompileTime: null,
                    lastCompileSuccess: false,
                    lastCompileErrors: [],
                };
                
                set({
                    documents: new Map(state.documents).set(id, doc),
                    activeDocumentId: id,
                });
                
                return doc;
            },
            
            closeDocument: (id: string) => {
                const state = get();
                const docs = new Map(state.documents);
                docs.delete(id);

                let newActiveId = state.activeDocumentId;
                if (newActiveId === id) {
                    newActiveId = docs.keys().next().value || null;
                }

                set({
                    documents: docs,
                    activeDocumentId: newActiveId,
                });

                // Clean up IndexedDB samples for this project
                sampleService.deleteProject(id).catch(console.error);
            },

            closeAllDocuments: () => {
                const state = get();
                const ids = Array.from(state.documents.keys());
                set({
                    documents: new Map(),
                    activeDocumentId: null,
                });
                // Clean up all associated sample data
                ids.forEach((id) => sampleService.deleteProject(id).catch(console.error));
            },
            
            setActiveDocument: (id: string) => {
                set({ activeDocumentId: id });
            },
            
            updateDocumentContent: (id: string, content: string) => {
                const state = get();
                const doc = state.documents.get(id);
                if (doc) {
                    const updatedDoc = { ...doc, content, isDirty: true };
                    set({
                        documents: new Map(state.documents).set(id, updatedDoc),
                    });
                }
            },
            
            updateDocumentFilename: (id: string, filename: string) => {
                const state = get();
                const doc = state.documents.get(id);
                if (doc) {
                    const updatedDoc = { ...doc, filename };
                    set({
                        documents: new Map(state.documents).set(id, updatedDoc),
                    });
                }
            },
            
            updateDocumentFileHandle: (id: string, handle: FileSystemFileHandle | undefined) => {
                const state = get();
                const doc = state.documents.get(id);
                if (doc) {
                    const updatedDoc = { ...doc, fileHandle: handle };
                    set({
                        documents: new Map(state.documents).set(id, updatedDoc),
                    });
                }
            },
            
            updateDocumentLanguage: (id: string, language: MMLLanguage) => {
                const state = get();
                const doc = state.documents.get(id);
                if (doc) {
                    const updatedDoc = { ...doc, language };
                    set({
                        documents: new Map(state.documents).set(id, updatedDoc),
                    });
                }
            },
            
            setDocumentDirty: (id: string, isDirty: boolean) => {
                const state = get();
                const doc = state.documents.get(id);
                if (doc) {
                    const updatedDoc = { ...doc, isDirty };
                    set({
                        documents: new Map(state.documents).set(id, updatedDoc),
                    });
                }
            },
            
            setCompileResults: (id: string, success: boolean, errors: CompileError[]) => {
                const state = get();
                const doc = state.documents.get(id);
                if (doc) {
                    const updatedDoc: Document = {
                        ...doc,
                        lastCompileTime: new Date(),
                        lastCompileSuccess: success,
                        lastCompileErrors: errors,
                        isDirty: false,
                    };
                    set({
                        documents: new Map(state.documents).set(id, updatedDoc),
                    });
                }
            },
            
            // ============================================================
            // Getters
            // ============================================================
            
            getActiveDocument: () => {
                const state = get();
                if (state.activeDocumentId) {
                    return state.documents.get(state.activeDocumentId) || null;
                }
                return null;
            },
            
            getDocument: (id: string) => {
                const state = get();
                return state.documents.get(id);
            },
            
            hasDirtyDocuments: () => {
                const state = get();
                for (const doc of state.documents.values()) {
                    if (doc.isDirty) {
                        return true;
                    }
                }
                return false;
            },
            
            getDocuments: () => {
                const state = get();
                return Array.from(state.documents.values());
            },
        }),
        {
            name: 'mml2vgm-document-store',
            storage: createJSONStorage(() => sessionStorage),
            partialize: (state) => ({
                documents: Array.from(state.documents.entries()),
                activeDocumentId: state.activeDocumentId,
                nextDocumentId: state.nextDocumentId,
            }),
            merge: (persistedState: any, currentState: any) => ({
                ...currentState,
                documents: new Map(persistedState.documents || []),
                activeDocumentId: persistedState.activeDocumentId || null,
                nextDocumentId: persistedState.nextDocumentId || 1,
            }),
        }
    )
);

// ============================================================================
// Selectors
// ============================================================================

// Selector for active document
export const selectActiveDocument = (state: DocumentStore) => 
    state.getActiveDocument();

// Selector for active document content
export const selectActiveDocumentContent = (state: DocumentStore) => {
    const doc = state.getActiveDocument();
    return doc?.content || '';
};

// Selector for active document ID
export const selectActiveDocumentId = (state: DocumentStore) => 
    state.activeDocumentId;

// Selector for all documents
export const selectDocuments = (state: DocumentStore) => 
    state.getDocuments();

// Selector for dirty state
export const selectHasDirtyDocuments = (state: DocumentStore) => 
    state.hasDirtyDocuments();

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Generate a unique document ID.
 */
export function generateDocumentId(): string {
    return `doc-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * Get language from file extension.
 */
export function getLanguageFromExtension(filename: string): MMLLanguage {
    const extension = filename.split('.').pop()?.toLowerCase();
    switch (extension) {
        case 'muc': return 'muc';
        case 'mml': return 'mml';
        case 'mdl': return 'mdl';
        case 'mus': return 'mus';
        default: return 'gwi';
    }
}

/**
 * Get file extension from language.
 */
export function getExtensionFromLanguage(language: MMLLanguage): string {
    switch (language) {
        case 'muc': return 'muc';
        case 'mml': return 'mml';
        case 'mdl': return 'mdl';
        case 'mus': return 'mus';
        default: return 'gwi';
    }
}
