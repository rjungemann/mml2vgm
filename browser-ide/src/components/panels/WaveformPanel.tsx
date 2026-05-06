import React from 'react';
import AudioWaveformView from '@/components/AudioWaveformView';

interface WaveformPanelProps {
  waveformSamples: number[];
}

const WaveformPanel: React.FC<WaveformPanelProps> = ({ waveformSamples }) => {
  return (
    <div style={{ padding: '8px 10px 10px' }}>
      <AudioWaveformView samples={waveformSamples} />
    </div>
  );
};

export default WaveformPanel;
