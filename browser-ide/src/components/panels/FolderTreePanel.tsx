import React, { useState, useEffect, useCallback } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { fileService } from '@/services/fileService';
import { useDocumentStore } from '@/stores/documentStore';
import type { TreeNode as FileServiceTreeNode } from '@/services/fileService';
import type { SoundChip } from '@/types';

// Tree node interface for UI
interface TreeNode {
  id: string;
  name: string;
  type: 'folder' | 'file' | 'chip' | 'channel';
  path: string;
  handle?: FileSystemFileHandle | FileSystemDirectoryHandle;
  children?: TreeNode[];
  icon?: string;
  isExpanded?: boolean;
  isSelected?: boolean;
  chip?: SoundChip;
  channel?: number;
}

const FolderTreePanel: React.FC = () => {
  const { createDocument, setActiveDocument } = useDocumentStore(
    useShallow((state) => ({
      createDocument: state.createDocument,
      setActiveDocument: state.setActiveDocument,
    }))
  );

  // State for tree data and loading
  const [tree, setTree] = useState<TreeNode[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Build tree from file service workspace
  const buildTreeFromFileService = useCallback((fileNode: FileServiceTreeNode): TreeNode => {
    // Map file service type to UI type
    const nodeType: TreeNode['type'] = 
      fileNode.type === 'directory' ? 'folder' : 'file';
    
    const node: TreeNode = {
      id: fileNode.id,
      name: fileNode.name,
      type: nodeType,
      path: fileNode.path || `/${fileNode.name}`,
      handle: fileNode.handle as FileSystemFileHandle | FileSystemDirectoryHandle | undefined,
      isExpanded: nodeType === 'folder' ? false : undefined,
      children: [],
    };

    if (fileNode.children && fileNode.children.length > 0) {
      node.children = fileNode.children.map(child => buildTreeFromFileService(child));
      node.isExpanded = false;
    }

    return node;
  }, []);

  // Refresh tree from file service
  const refreshTree = useCallback(async () => {
    const workspace = fileService.getCurrentWorkspace();

    if (workspace && workspace.root) {
      setIsLoading(true);
      setError(null);

      try {
        const filteredRoot = fileService.filterMMLFiles(workspace.root);
        const newTree: TreeNode[] = [buildTreeFromFileService(filteredRoot)];
        setTree(newTree);
      } catch (err) {
        setError(`Failed to load workspace: ${err}`);
        // Fall back to empty tree
        setTree([]);
      } finally {
        setIsLoading(false);
      }
    } else {
      // No workspace open - show default tree
      setTree([
        {
          id: 'root',
          name: 'Workspace',
          type: 'folder',
          path: '/',
          isExpanded: true,
          children: [],
        },
      ]);
    }
  }, [buildTreeFromFileService]);

  // Subscribe to file service state changes
  useEffect(() => {
    const handleStateUpdate = () => {
      refreshTree();
    };

    fileService.addStateListener(handleStateUpdate);

    // Initial refresh
    refreshTree();

    return () => {
      fileService.removeStateListener(handleStateUpdate);
    };
  }, [refreshTree]);

  // Toggle node expansion
  const toggleExpand = (nodeId: string) => {
    const updateTree = (nodes: TreeNode[]): TreeNode[] => {
      return nodes.map((node) => {
        if (node.id === nodeId) {
          return { ...node, isExpanded: !node.isExpanded };
        }
        if (node.children) {
          return { ...node, children: updateTree(node.children) };
        }
        return node;
      });
    };
    
    setTree(updateTree(tree));
  };

  // Select a node
  const selectNode = (nodeId: string) => {
    const updateTree = (nodes: TreeNode[]): TreeNode[] => {
      return nodes.map((node) => {
        const updatedNode = { ...node, isSelected: node.id === nodeId };
        if (node.children) {
          return { ...updatedNode, children: updateTree(node.children) };
        }
        return updatedNode;
      });
    };
    
    setTree(updateTree(tree));
  };

  // Convert UI TreeNode to FileService TreeNode
  const toFileServiceNode = useCallback((node: TreeNode): FileServiceTreeNode => {
    return {
      id: node.id,
      name: node.name,
      path: node.path,
      type: node.type === 'folder' ? 'directory' : 'file',
      handle: node.handle as FileSystemFileHandle | FileSystemDirectoryHandle | undefined,
    };
  }, []);

  // Handle file open from tree
  const handleOpenFile = useCallback(async (node: TreeNode) => {
    if (node.type !== 'file' || !node.handle) return;

    try {
      const fileNode = toFileServiceNode(node);
      const content = await fileService.openFileFromTree(fileNode);
      if (content) {
        // Create a new document with the content
        const language = fileService.detectLanguage(node.name);
        const doc = createDocument(language as any);
        // Update the new document with file content
        useDocumentStore.getState().updateDocumentContent(doc.id, content);
        useDocumentStore.getState().updateDocumentFilename(doc.id, node.name);
        useDocumentStore.getState().setDocumentDirty(doc.id, false);
        setActiveDocument(doc.id);
      }
    } catch (err) {
      setError(`Failed to open file: ${err}`);
    }
  }, [createDocument, setActiveDocument, toFileServiceNode]);

  // Handle open workspace
  const handleOpenWorkspace = useCallback(async () => {
    await fileService.openWorkspace();
  }, []);

  // Handle refresh
  const handleRefresh = useCallback(async () => {
    await fileService.refreshWorkspace();
  }, []);

  // Render a tree node
  const renderNode = (node: TreeNode, depth = 0) => {
    const hasChildren = node.children && node.children.length > 0;
    const isExpanded = node.isExpanded ?? (node.type === 'folder' ? true : false);
    const isSelected = node.isSelected ?? false;

    return (
      <div key={node.id} style={{ marginLeft: `${depth * 16}px` }}>
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            height: '22px',
            padding: '0 4px',
            cursor: 'pointer',
            borderRadius: '2px',
            backgroundColor: isSelected ? 'var(--highlight-line)' : 'transparent',
          }}
          onClick={async () => {
            if (node.type === 'file') {
              await handleOpenFile(node);
            }
            selectNode(node.id);
          }}
          onDoubleClick={() => {
            if (hasChildren) {
              toggleExpand(node.id);
            }
          }}
        >
          {/* Expansion toggle */}
          {hasChildren && (
            <span
              style={{
                width: '16px',
                textAlign: 'center',
                fontSize: '11px',
                userSelect: 'none',
              }}
              onClick={(e) => {
                e.stopPropagation();
                toggleExpand(node.id);
              }}
            >
              {isExpanded ? '▼' : '▶'}
            </span>
          )}
          
          {!hasChildren && <span style={{ width: '16px' }} />}
          
          {/* Icon */}
          <span style={{ width: '16px', textAlign: 'center' }}>
            {getIcon(node.type, node)}
          </span>
          
          {/* Name */}
          <span 
            style={{
              fontSize: '12px',
              color: node.type === 'chip' ? 'var(--accent-primary)' : 
                     node.type === 'channel' ? 'var(--accent-secondary)' : 
                     (node.type === 'file' ? 'var(--text-primary)' : 'var(--text-secondary)'),
            }}
          >
            {node.name}
          </span>
        </div>
        
        {/* Children */}
        {hasChildren && isExpanded && (
          <div>
            {node.children?.map((child) => renderNode(child, depth + 1))}
          </div>
        )}
      </div>
    );
  };

  // Get icon for node type
  const getIcon = (type: string, node?: TreeNode): string => {
    switch (type) {
      case 'folder':
        return node?.isExpanded ? '📂' : '📁';
      case 'file':
        return '📄';
      case 'chip':
        return '🎛️';
      case 'channel':
        return '🔊';
      default:
        return '';
    }
  };

  // Count all files in tree
  const countItems = (nodes: TreeNode[]): number => {
    let count = 0;
    for (const node of nodes) {
      if (node.type === 'file') {
        count++;
      }
      if (node.children) {
        count += countItems(node.children);
      }
    }
    return count;
  };

  const totalItems = countItems(tree);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
      {/* Header with controls */}
      <div 
        style={{
          display: 'flex',
          alignItems: 'center',
          height: '28px',
          padding: '0 4px',
          borderBottom: '1px solid var(--border-color)',
          backgroundColor: 'var(--background-secondary)',
        }}
      >
        <button 
          className="button small"
          onClick={handleOpenWorkspace}
          disabled={isLoading || !fileService.isSupported()}
          title={!fileService.isSupported() ? 'File System Access API not supported in this browser' : 'Open Workspace'}
        >
          📁 Open
        </button>
        <button className="button small" onClick={handleRefresh} disabled={isLoading || !fileService.getCurrentWorkspace()}>
          🔄 Refresh
        </button>
        <div style={{ flex: 1 }} />
        {error && (
          <span style={{ fontSize: '11px', color: 'var(--error)' }}>
            {error}
          </span>
        )}
        {isLoading && (
          <span style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
            Loading...
          </span>
        )}
      </div>

      {/* Tree view */}
      <div style={{ flex: 1, overflowY: 'auto', padding: '4px' }}>
        {tree.length === 0 && !isLoading && (
          <div style={{ padding: '8px', color: 'var(--text-muted)', fontSize: '12px', textAlign: 'center' }}>
            No workspace open. Click "Open" to select a folder.
          </div>
        )}
        {tree.map((node) => renderNode(node))}
      </div>

      {/* Footer with info */}
      <div 
        style={{
          height: '20px',
          padding: '0 8px',
          borderTop: '1px solid var(--border-color)',
          fontSize: '11px',
          color: 'var(--text-muted)',
        }}
      >
        {totalItems} item{totalItems !== 1 ? 's' : ''} {fileService.getCurrentWorkspace()?.name ? `in ${fileService.getCurrentWorkspace()?.name}` : ''}
      </div>
    </div>
  );
};

export default FolderTreePanel;
