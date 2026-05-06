import React, { useEffect, useRef } from 'react';

interface AudioWaveformViewProps {
  samples: number[];
  height?: number;
}

const AudioWaveformView: React.FC<AudioWaveformViewProps> = ({
  samples,
  height = 96,
}) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const dpr = window.devicePixelRatio || 1;
    const cssWidth = canvas.clientWidth || 640;
    const cssHeight = height;

    canvas.width = Math.floor(cssWidth * dpr);
    canvas.height = Math.floor(cssHeight * dpr);

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, cssWidth, cssHeight);

    ctx.fillStyle = 'var(--bg-secondary)';
    ctx.fillRect(0, 0, cssWidth, cssHeight);

    const centerY = cssHeight / 2;
    ctx.strokeStyle = 'var(--border-color)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, centerY);
    ctx.lineTo(cssWidth, centerY);
    ctx.stroke();

    if (samples.length === 0) {
      return;
    }

    ctx.strokeStyle = 'var(--accent-color, #4ea1ff)';
    ctx.lineWidth = 1.5;
    ctx.beginPath();

    const maxAmplitude = cssHeight * 0.45;
    const step = samples.length > 1 ? cssWidth / (samples.length - 1) : cssWidth;

    for (let i = 0; i < samples.length; i++) {
      const x = i * step;
      const y = centerY - Math.max(-1, Math.min(1, samples[i])) * maxAmplitude;
      if (i === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    }

    ctx.stroke();
  }, [samples, height]);

  return (
    <canvas
      ref={canvasRef}
      aria-label="Audio waveform"
      style={{
        width: '100%',
        height: `${height}px`,
        border: '1px solid var(--border-color)',
        borderRadius: '4px',
        background: 'var(--bg-secondary)',
        display: 'block',
      }}
    />
  );
};

export default AudioWaveformView;
