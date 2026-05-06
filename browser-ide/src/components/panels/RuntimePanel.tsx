import React, { useEffect, useState } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { useCompileStore } from '@/stores/compileStore';
import { useDocumentStore } from '@/stores/documentStore';
import { audioService, type AudioRuntimeDebugInfo } from '@/services/audioService';

interface RuntimePanelProps {
  audioRuntimeDebug: AudioRuntimeDebugInfo;
}

const RuntimePanel: React.FC<RuntimePanelProps> = ({ audioRuntimeDebug }) => {
  const { getResult } = useCompileStore(
    useShallow((state) => ({
      getResult: state.getResult,
    }))
  );

  const { activeDocumentId } = useDocumentStore(
    useShallow((state) => ({
      activeDocumentId: state.activeDocumentId,
    }))
  );

  // Get active compile result
  const activeResult = activeDocumentId ? getResult(activeDocumentId) : null;

  return (
    <div style={{ padding: '8px 10px 10px', display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '6px 12px', color: 'var(--text-primary)' }}>
      <div>Compile bytes: {activeResult?.data?.length || 0}</div>
      <div>Compile parts: {activeResult?.partCount || 0}</div>
      <div>Compile commands: {activeResult?.commandCount || 0}</div>
      <div>Compile chips: {(activeResult?.chipsUsed || []).join(', ') || '(none)'}</div>

      <div>Audio playing: {audioRuntimeDebug.isPlaying ? 'yes' : 'no'}</div>
      <div>Audio paused: {audioRuntimeDebug.isPaused ? 'yes' : 'no'}</div>
      <div>VGM bytes loaded: {audioRuntimeDebug.vgmDataLength}</div>
      <div>Parsed VGM commands: {audioRuntimeDebug.parsedCommandCount}</div>
      <div>Commands applied: {audioRuntimeDebug.appliedWriteCount}</div>
      <div>Commands skipped: {audioRuntimeDebug.skippedWriteCount}</div>
      <div>Pending commands: {audioRuntimeDebug.pendingCommandCount}</div>
      <div>Buffers generated: {audioRuntimeDebug.generatedBufferCount}</div>
      <div>Silent buffer streak: {audioRuntimeDebug.silentBufferCount}</div>
      <div>Last peak: {audioRuntimeDebug.lastPeak.toExponential(3)}</div>
      <div>Silence warning emitted: {audioRuntimeDebug.emittedSilenceWarning ? 'yes' : 'no'}</div>
      <div>Playback chips: {audioRuntimeDebug.chips.join(', ') || '(none)'}</div>
    </div>
  );
};

export default RuntimePanel;
