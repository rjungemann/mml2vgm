import React, { useState, useEffect, useCallback } from 'react';
import { scriptService, SCRIPT_TEMPLATES, type Script, type ScriptFunction } from '@/services/scriptService';
import type { MMLLanguage } from '@/types';
import { useSessionStorageState } from '@/utils/useSessionStorageState';

interface ScriptPanelProps {
    documentId?: string;
    documentContent?: string;
    documentLanguage?: MMLLanguage;
}

const ScriptPanel: React.FC<ScriptPanelProps> = ({
    documentId,
    documentContent = '',
    documentLanguage = 'gwi',
}) => {
    // State
    const [scripts, setScripts] = useState<Script[]>([]);
    const [currentScript, setCurrentScript] = useState<Script | null>(null);
    const [functions, setFunctions] = useState<ScriptFunction[]>([]);
    const [isExecuting, setIsExecuting] = useState(false);
    const [executionResult, setExecutionResult] = useState<{
        success: boolean;
        output: string | null;
        error: string | null;
        time: number;
    } | null>(null);
    const [isPyodideReady, setIsPyodideReady] = useState(false);
    const [isPyodideSupported, setIsPyodideSupported] = useState(true);
    const [showTemplates, setShowTemplates] = useSessionStorageState<boolean>('mml2vgm:script:showTemplates', false);
    
    // Load scripts on mount
    useEffect(() => {
        const unsubscribe = scriptService.subscribe((state) => {
            setScripts(state.scripts);
            setIsPyodideReady(state.isInitialized);
            setIsPyodideSupported(state.isSupported);
            setIsExecuting(state.isExecuting);
        });
        
        // Load scripts
        setScripts(scriptService.getAllScripts());
        setIsPyodideReady(scriptService.isInitialized);
        setIsPyodideSupported(scriptService.isSupported);
        
        return () => unsubscribe();
    }, []);
    
    // Update current script
    useEffect(() => {
        const current = scriptService.getCurrentScript();
        setCurrentScript(current);
        if (current) {
            analyzeScriptFunctions(current);
        }
    }, [scripts]);
    
    // Analyze script functions
    const analyzeScriptFunctions = useCallback(async (script: Script) => {
        try {
            const funcs = await scriptService.getScriptFunctions(script);
            setFunctions(funcs);
        } catch (error) {
            console.error('Failed to analyze script functions:', error);
            setFunctions([]);
        }
    }, []);
    
    // Handle script selection
    const handleSelectScript = useCallback((script: Script | null) => {
        scriptService.setCurrentScript(script?.id || null);
        setCurrentScript(script);
        if (script) {
            analyzeScriptFunctions(script);
        } else {
            setFunctions([]);
        }
    }, [analyzeScriptFunctions]);
    
    // Handle new script from template
    const handleNewFromTemplate = useCallback((templateKey: string) => {
        const template = (SCRIPT_TEMPLATES as any)[templateKey];
        if (!template) return;
        
        const script = scriptService.createScript(
            template.name,
            template.content,
            null
        );
        scriptService.setCurrentScript(script.id);
        setCurrentScript(script);
        setShowTemplates(false);
    }, []);
    
    // Handle new blank script
    const handleNewScript = useCallback(() => {
        const script = scriptService.createScript(
            'Untitled Script',
            '# New Python Script\n# Use document_content, document_language for access\n\ndef main():\n    pass\n',
            null
        );
        scriptService.setCurrentScript(script.id);
        setCurrentScript(script);
    }, []);
    
    // Handle script content change
    const handleContentChange = useCallback((content: string) => {
        if (currentScript) {
            scriptService.updateScriptContent(currentScript.id, content);
            // Re-analyze functions after a brief delay
            setTimeout(() => analyzeScriptFunctions({ ...currentScript, content }), 500);
        }
    }, [currentScript, analyzeScriptFunctions]);
    
    // Handle save script
    const handleSaveScript = useCallback(() => {
        if (currentScript) {
            scriptService.saveScript(currentScript);
        }
    }, [currentScript]);
    
    // Handle delete script
    const handleDeleteScript = useCallback((id: string) => {
        if (window.confirm('Delete this script?')) {
            scriptService.deleteScript(id);
        }
    }, []);
    
    // Handle execute script
    const handleExecuteScript = useCallback(async () => {
        if (!currentScript || !isPyodideReady) return;
        
        setIsExecuting(true);
        setExecutionResult(null);
        
        try {
            const startTime = performance.now();
            const result = await scriptService.executeScript(currentScript, {
                documentContent,
                documentLanguage,
            });
            
            const endTime = performance.now();
            
            setExecutionResult({
                success: result.success,
                output: result.output,
                error: result.error,
                time: endTime - startTime,
            });
        } catch (error) {
            setExecutionResult({
                success: false,
                output: null,
                error: error instanceof Error ? error.message : String(error),
                time: performance.now() - performance.now(),
            });
        } finally {
            setIsExecuting(false);
        }
    }, [currentScript, isPyodideReady, documentContent, documentLanguage]);
    
    // Handle execute function
    const handleExecuteFunction = useCallback(async (func: ScriptFunction) => {
        if (!currentScript || !isPyodideReady) return;
        
        setIsExecuting(true);
        setExecutionResult(null);
        
        try {
            const startTime = performance.now();
            const result = await scriptService.executeFunction(currentScript, func.name);
            
            const endTime = performance.now();
            
            setExecutionResult({
                success: result.success,
                output: result.output,
                error: result.error,
                time: endTime - startTime,
            });
        } catch (error) {
            setExecutionResult({
                success: false,
                output: null,
                error: error instanceof Error ? error.message : String(error),
                time: performance.now() - performance.now(),
            });
        } finally {
            setIsExecuting(false);
        }
    }, [currentScript, isPyodideReady]);
    
    // Initialize Pyodide on first render
    useEffect(() => {
        if (isPyodideSupported && !isPyodideReady) {
            scriptService.init();
        }
    }, [isPyodideSupported, isPyodideReady]);
    
    // Render
    return (
        <div style={styles.panel}>
            {/* Header */}
            <div style={styles.header}>
                <span style={styles.title}>Python Scripts</span>
                <div style={styles.headerActions}>
                    {!isPyodideSupported && (
                        <span style={styles.warning} title="Pyodide not supported in this browser">
                            ⚠️ Python not available
                        </span>
                    )}
                    {!isPyodideReady && isPyodideSupported && (
                        <span style={styles.info} title="Initializing Pyodide...">
                            Initializing Python...
                        </span>
                    )}
                    <button 
                        onClick={() => setShowTemplates(!showTemplates)}
                        style={styles.button}
                        disabled={!isPyodideReady}
                    >
                        New Script ▼
                    </button>
                </div>
            </div>
            
            <div style={styles.container}>
                {/* Script List */}
                <div style={styles.sidebar}>
                    <div style={styles.sectionTitle}>Scripts</div>
                    {scripts.length === 0 ? (
                        <div style={styles.emptyState}>
                            No scripts yet
                        </div>
                    ) : (
                        <ul style={styles.list}>
                            {scripts.map((script) => (
                                <li 
                                    key={script.id}
                                    style={{
                                        ...styles.listItem,
                                        background: currentScript?.id === script.id 
                                            ? 'var(--selection-bg)' 
                                            : 'transparent',
                                    }}
                                    onClick={() => handleSelectScript(script)}
                                >
                                    <div style={styles.listItemContent}>
                                        <span style={styles.listItemName}>{script.name}</span>
                                        {script.isDirty && (
                                            <span style={styles.dirtyIndicator}>•</span>
                                        )}
                                    </div>
                                    <button 
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            handleDeleteScript(script.id);
                                        }}
                                        style={styles.deleteButton}
                                        title="Delete"
                                    >
                                        ×
                                    </button>
                                </li>
                            ))}
                        </ul>
                    )}
                </div>
                
                {/* Editor Area */}
                <div style={styles.editorArea}>
                    {currentScript ? (
                        <>
                            {/* Script Toolbar */}
                            <div style={styles.toolbar}>
                                <button 
                                    onClick={handleExecuteScript}
                                    style={styles.toolbarButton}
                                    disabled={!isPyodideReady || isExecuting}
                                >
                                    {isExecuting ? 'Executing...' : '▶ Run'}
                                </button>
                                <button 
                                    onClick={handleSaveScript}
                                    style={styles.toolbarButton}
                                >
                                    💾 Save
                                </button>
                                <span style={styles.toolbarSpacer} />
                                {currentScript.lastExecuted && (
                                    <span style={styles.lastExecuted}>
                                        Last run: {new Date(currentScript.lastExecuted).toLocaleTimeString()}
                                    </span>
                                )}
                            </div>
                            
                            {/* Script Name */}
                            <div style={styles.scriptHeader}>
                                <input
                                    type="text"
                                    value={currentScript.name}
                                    onChange={(e) => {
                                        setCurrentScript({
                                            ...currentScript,
                                            name: e.target.value,
                                        });
                                    }}
                                    style={styles.scriptNameInput}
                                />
                            </div>
                            
                            {/* Script Content */}
                            <textarea
                                value={currentScript.content}
                                onChange={(e) => handleContentChange(e.target.value)}
                                style={styles.scriptEditor}
                                spellCheck={false}
                                placeholder="# Write your Python script here..."
                            />
                            
                            {/* Functions List */}
                            {functions.length > 0 && (
                                <div style={styles.functionsSection}>
                                    <div style={styles.sectionTitle}>Functions</div>
                                    <div style={styles.functionsList}>
                                        {functions.map((func) => (
                                            <button
                                                key={func.name}
                                                onClick={() => handleExecuteFunction(func)}
                                                style={styles.functionButton}
                                                title={`${func.name}(${func.parameters.map(p => p.name).join(', ')})`}
                                            >
                                                {func.name}(
                                                {func.parameters.length > 0 && (
                                                    <>
                                                        {func.parameters.map((p, i) => (
                                                            <React.Fragment key={p.name}>
                                                                {i > 0 && ', '}
                                                                <em>{p.name}</em>
                                                            </React.Fragment>
                                                        ))}
                                                    </>
                                                )}
                                                )
                                            </button>
                                        ))}
                                    </div>
                                </div>
                            )}
                        </>
                    ) : (
                        <div style={styles.emptyEditor}>
                            <p>No script selected</p>
                            <p style={styles.muted}>Create a new script or select from the list</p>
                        </div>
                    )}
                </div>
            </div>
            
            {/* Output Area */}
            {executionResult && (
                <div style={styles.outputArea}>
                    <div style={styles.outputHeader}>
                        <span>Output</span>
                        <span style={{
                            color: executionResult.success ? 'var(--success-color)' : 'var(--error-color)',
                            marginLeft: '10px',
                        }}>
                            {executionResult.success ? '✓ Success' : '✗ Error'}
                            ({executionResult.time.toFixed(1)}ms)
                        </span>
                    </div>
                    <pre style={styles.outputContent}>
                        {executionResult.output}
                        {executionResult.error && (
                            <div style={styles.errorContent}>{executionResult.error}</div>
                        )}
                    </pre>
                </div>
            )}
            
            {/* New Script Menu */}
            {showTemplates && (
                <div style={styles.modalOverlay} onClick={() => setShowTemplates(false)}>
                    <div style={styles.modal} onClick={(e) => e.stopPropagation()}>
                        <div style={styles.modalHeader}>
                            <span>New Script</span>
                            <button onClick={() => setShowTemplates(false)} style={styles.modalClose}>
                                ×
                            </button>
                        </div>
                        <div style={styles.modalContent}>
                            <button 
                                onClick={() => handleNewScript()}
                                style={styles.templateButton}
                            >
                                Blank Script
                            </button>
                            {Object.entries(SCRIPT_TEMPLATES).map(([key, template]) => (
                                <button
                                    key={key}
                                    onClick={() => handleNewFromTemplate(key)}
                                    style={styles.templateButton}
                                >
                                    {template.name}
                                </button>
                            ))}
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};

// Styles
const styles = {
    panel: {
        display: 'flex',
        flexDirection: 'column' as const,
        height: '100%',
        background: 'var(--editor-bg)',
        color: 'var(--text)',
        fontFamily: 'var(--font-family)',
    },
    header: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '4px 8px',
        borderBottom: '1px solid var(--border-color)',
        fontSize: '11px',
    },
    title: {
        fontWeight: 'bold' as const,
    },
    headerActions: {
        display: 'flex',
        gap: '8px',
        alignItems: 'center',
    },
    button: {
        fontSize: '11px',
        padding: '2px 6px',
        background: 'var(--button-bg)',
        color: 'var(--button-fg)',
        border: '1px solid var(--border-color)',
        borderRadius: '3px',
        cursor: 'pointer',
    },
    warning: {
        color: 'var(--warning-color)',
        fontSize: '11px',
    },
    info: {
        color: 'var(--info-color)',
        fontSize: '11px',
    },
    container: {
        display: 'flex',
        flex: 1,
        overflow: 'hidden',
    },
    sidebar: {
        width: '150px',
        borderRight: '1px solid var(--border-color)',
        overflowY: 'auto' as const,
        background: 'var(--sidebar-bg)',
    },
    sectionTitle: {
        padding: '4px 8px',
        fontSize: '10px',
        fontWeight: 'bold' as const,
        color: 'var(--text-muted)',
        textTransform: 'uppercase' as const,
        borderBottom: '1px solid var(--border-color)',
    },
    list: {
        listStyle: 'none',
        padding: 0,
        margin: 0,
    },
    listItem: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '4px 8px',
        cursor: 'pointer',
        fontSize: '11px',
        borderBottom: '1px solid transparent',
        ':hover': {
            background: 'var(--hover-bg)',
        },
    },
    listItemContent: {
        display: 'flex',
        alignItems: 'center',
        gap: '4px',
    },
    listItemName: {
        whiteSpace: 'nowrap' as const,
        overflow: 'hidden',
        textOverflow: 'ellipsis' as const,
    },
    dirtyIndicator: {
        color: 'var(--modified-color)',
        fontSize: '8px',
    },
    deleteButton: {
        background: 'transparent',
        border: 'none',
        color: 'var(--text-muted)',
        cursor: 'pointer',
        fontSize: '10px',
        padding: '2px',
    },
    emptyState: {
        padding: '16px',
        textAlign: 'center' as const,
        color: 'var(--text-muted)',
        fontSize: '11px',
    },
    editorArea: {
        flex: 1,
        display: 'flex',
        flexDirection: 'column' as const,
        overflow: 'hidden',
    },
    toolbar: {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        padding: '4px 8px',
        borderBottom: '1px solid var(--border-color)',
        background: 'var(--toolbar-bg)',
    },
    toolbarButton: {
        fontSize: '11px',
        padding: '2px 6px',
        background: 'var(--button-bg)',
        color: 'var(--button-fg)',
        border: '1px solid var(--border-color)',
        borderRadius: '3px',
        cursor: 'pointer',
    },
    toolbarSpacer: {
        flex: 1,
    },
    lastExecuted: {
        fontSize: '10px',
        color: 'var(--text-muted)',
    },
    scriptHeader: {
        padding: '4px 8px',
    },
    scriptNameInput: {
        width: '100%',
        padding: '4px',
        fontSize: '12px',
        background: 'var(--input-bg)',
        color: 'var(--input-fg)',
        border: '1px solid var(--border-color)',
        borderRadius: '3px',
    },
    scriptEditor: {
        flex: 1,
        padding: '8px',
        fontFamily: 'Consolas, Monaco, monospace',
        fontSize: '12px',
        background: 'var(--editor-bg)',
        color: 'var(--editor-fg)',
        border: 'none',
        outline: 'none',
        resize: 'none' as const,
        overflow: 'auto' as const,
    },
    emptyEditor: {
        flex: 1,
        display: 'flex',
        flexDirection: 'column' as const,
        justifyContent: 'center',
        alignItems: 'center',
        color: 'var(--text-muted)',
        padding: '32px',
    },
    muted: {
        color: 'var(--text-muted)',
        fontSize: '11px',
    },
    functionsSection: {
        padding: '8px',
        borderTop: '1px solid var(--border-color)',
    },
    functionsList: {
        display: 'flex',
        flexWrap: 'wrap' as const,
        gap: '4px',
    },
    functionButton: {
        fontSize: '10px',
        padding: '2px 4px',
        background: 'var(--button-bg)',
        color: 'var(--button-fg)',
        border: '1px solid var(--border-color)',
        borderRadius: '3px',
        cursor: 'pointer',
    },
    outputArea: {
        borderTop: '1px solid var(--border-color)',
        background: 'var(--output-bg)',
    },
    outputHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '4px 8px',
        fontSize: '11px',
        borderBottom: '1px solid var(--border-color)',
    },
    outputContent: {
        padding: '8px',
        fontSize: '11px',
        fontFamily: 'Consolas, Monaco, monospace',
        whiteSpace: 'pre-wrap' as const,
        wordBreak: 'break-all' as const,
        maxHeight: '200px',
        overflow: 'auto' as const,
    },
    errorContent: {
        color: 'var(--error-color)',
    },
    modalOverlay: {
        position: 'fixed' as const,
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        background: 'rgba(0, 0, 0, 0.5)',
        display: 'flex',
        justifyContent: 'center' as const,
        alignItems: 'center' as const,
        zIndex: 1000,
    },
    modal: {
        background: 'var(--modal-bg)',
        border: '1px solid var(--border-color)',
        borderRadius: '4px',
        padding: '16px',
        minWidth: '200px',
    },
    modalHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '8px',
        paddingBottom: '8px',
        borderBottom: '1px solid var(--border-color)',
    },
    modalClose: {
        background: 'transparent',
        border: 'none',
        color: 'var(--text-muted)',
        cursor: 'pointer',
        fontSize: '16px',
    },
    modalContent: {
        display: 'flex',
        flexDirection: 'column' as const,
        gap: '4px',
    },
    templateButton: {
        padding: '6px 12px',
        fontSize: '11px',
        background: 'var(--button-bg)',
        color: 'var(--button-fg)',
        border: '1px solid var(--border-color)',
        borderRadius: '3px',
        cursor: 'pointer',
        textAlign: 'left' as const,
    },
};

export default ScriptPanel;
