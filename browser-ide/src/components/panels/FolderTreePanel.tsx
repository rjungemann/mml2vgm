import React, { useState } from 'react';
import { SoundChip } from '@/types';

// Tree node interface
interface TreeNode {
  id: string;
  name: string;
  type: 'folder' | 'file' | 'chip' | 'channel';
  children?: TreeNode[];
  icon?: string;
  isExpanded?: boolean;
  isSelected?: boolean;
  chip?: SoundChip;
  channel?: number;
}

const FolderTreePanel: React.FC = () => {
  // Build the folder tree structure
  const [tree, setTree] = useState<TreeNode[]>(() => [
    {
      id: 'root',
      name: 'Project',
      type: 'folder',
      isExpanded: true,
      children: [
        {
          id: 'songs',
          name: 'Songs',
          type: 'folder',
          isExpanded: true,
          children: [
            { id: 'song1', name: 'Song1.mml', type: 'file' },
            { id: 'song2', name: 'Song2.mml', type: 'file' },
          ],
        },
        {
          id: 'chips',
          name: 'Sound Chips',
          type: 'folder',
          isExpanded: true,
          children: [
            {
              id: 'ym2608',
              name: 'YM2608 (OPNA)',
              type: 'chip',
              chip: 'YM2608',
              children: [
                { id: 'ym2608-fm1', name: 'FM Channel 1', type: 'channel', chip: 'YM2608', channel: 0 },
                { id: 'ym2608-fm2', name: 'FM Channel 2', type: 'channel', chip: 'YM2608', channel: 1 },
                { id: 'ym2608-fm3', name: 'FM Channel 3', type: 'channel', chip: 'YM2608', channel: 2 },
                { id: 'ym2608-fm4', name: 'FM Channel 4', type: 'channel', chip: 'YM2608', channel: 3 },
                { id: 'ym2608-fm5', name: 'FM Channel 5', type: 'channel', chip: 'YM2608', channel: 4 },
                { id: 'ym2608-fm6', name: 'FM Channel 6', type: 'channel', chip: 'YM2608', channel: 5 },
              ],
            },
            {
              id: 'ssg',
              name: 'AY8910 (SSG)',
              type: 'chip',
              chip: 'AY8910',
              children: [
                { id: 'ssg-ch1', name: 'SSG Channel A', type: 'channel', chip: 'AY8910', channel: 0 },
                { id: 'ssg-ch2', name: 'SSG Channel B', type: 'channel', chip: 'AY8910', channel: 1 },
                { id: 'ssg-ch3', name: 'SSG Channel C', type: 'channel', chip: 'AY8910', channel: 2 },
              ],
            },
            {
              id: 'rhythm',
              name: 'Rhythm (ADPCM)',
              type: 'chip',
              chip: 'YM2608',
              children: [
                { id: 'rhythm-ch', name: 'Rhythm Channel', type: 'channel', chip: 'YM2608', channel: 6 },
                { id: 'adpcm-ch', name: 'ADPCM Channel', type: 'channel', chip: 'YM2608', channel: 7 },
              ],
            },
          ],
        },
      ],
    },
  ]);

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

  // Render a tree node
  const renderNode = (node: TreeNode, depth = 0) => {
    const hasChildren = node.children && node.children.length > 0;
    const isExpanded = node.isExpanded ?? true;
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
          onClick={() => {
            if (hasChildren) {
              toggleExpand(node.id);
            }
            selectNode(node.id);
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
                     'var(--text-primary)',
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
        }}
      >
        <button className="button small">
          📁 Open
        </button>
        <button className="button small">
          💾 Save
        </button>
        <div style={{ flex: 1 }} />
        <button className="button small">
          🔄 Refresh
        </button>
      </div>

      {/* Tree view */}
      <div style={{ flex: 1, overflowY: 'auto', padding: '4px' }}>
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
        {tree[0]?.children?.length || 0} items
      </div>
    </div>
  );
};

export default FolderTreePanel;
