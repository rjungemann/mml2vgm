import fs from 'node:fs';
import path from 'node:path';

function usage() {
  console.error('Usage: node scripts/compare_wav.mjs <reference-dir> <current-dir>');
}

function walkWavs(dir) {
  const out = [];
  if (!fs.existsSync(dir)) {
    return out;
  }

  const stack = [dir];
  while (stack.length > 0) {
    const current = stack.pop();
    const entries = fs.readdirSync(current, { withFileTypes: true });

    for (const entry of entries) {
      const full = path.join(current, entry.name);
      if (entry.isDirectory()) {
        stack.push(full);
      } else if (entry.isFile() && entry.name.toLowerCase().endsWith('.wav')) {
        out.push(full);
      }
    }
  }

  return out.sort();
}

function readWav(filePath) {
  const buf = fs.readFileSync(filePath);

  if (buf.length < 44) {
    throw new Error(`Invalid WAV (too short): ${filePath}`);
  }
  if (buf.toString('ascii', 0, 4) !== 'RIFF' || buf.toString('ascii', 8, 12) !== 'WAVE') {
    throw new Error(`Invalid WAV header: ${filePath}`);
  }

  let offset = 12;
  let fmt = null;
  let dataOffset = -1;
  let dataSize = -1;

  while (offset + 8 <= buf.length) {
    const chunkId = buf.toString('ascii', offset, offset + 4);
    const chunkSize = buf.readUInt32LE(offset + 4);
    const chunkDataStart = offset + 8;

    if (chunkDataStart + chunkSize > buf.length) {
      throw new Error(`Corrupt WAV chunk ${chunkId} in ${filePath}`);
    }

    if (chunkId === 'fmt ') {
      if (chunkSize < 16) {
        throw new Error(`Unsupported fmt chunk size ${chunkSize} in ${filePath}`);
      }
      fmt = {
        audioFormat: buf.readUInt16LE(chunkDataStart),
        channels: buf.readUInt16LE(chunkDataStart + 2),
        sampleRate: buf.readUInt32LE(chunkDataStart + 4),
        bitsPerSample: buf.readUInt16LE(chunkDataStart + 14),
      };
    } else if (chunkId === 'data') {
      dataOffset = chunkDataStart;
      dataSize = chunkSize;
      break;
    }

    // Chunks are word-aligned.
    offset = chunkDataStart + chunkSize + (chunkSize % 2);
  }

  if (!fmt) {
    throw new Error(`Missing fmt chunk in ${filePath}`);
  }
  if (dataOffset < 0 || dataSize < 0) {
    throw new Error(`Missing data chunk in ${filePath}`);
  }
  if (fmt.audioFormat !== 1) {
    throw new Error(`Unsupported WAV format ${fmt.audioFormat} (expected PCM=1) in ${filePath}`);
  }
  if (fmt.bitsPerSample !== 16) {
    throw new Error(`Unsupported bit depth ${fmt.bitsPerSample} (expected 16) in ${filePath}`);
  }

  const bytesPerSample = fmt.bitsPerSample / 8;
  const frameSize = bytesPerSample * fmt.channels;
  if (frameSize <= 0) {
    throw new Error(`Invalid frame size in ${filePath}`);
  }

  const frames = Math.floor(dataSize / frameSize);
  const channelData = Array.from({ length: fmt.channels }, () => new Int16Array(frames));

  let pos = dataOffset;
  for (let i = 0; i < frames; i += 1) {
    for (let c = 0; c < fmt.channels; c += 1) {
      channelData[c][i] = buf.readInt16LE(pos);
      pos += 2;
    }
  }

  return {
    sampleRate: fmt.sampleRate,
    channels: fmt.channels,
    bitsPerSample: fmt.bitsPerSample,
    frames,
    channelData,
  };
}

function isSilent(channelData) {
  for (const channel of channelData) {
    for (let i = 0; i < channel.length; i += 1) {
      if (channel[i] !== 0) {
        return false;
      }
    }
  }
  return true;
}

function comparePcm(reference, current) {
  const frameCount = Math.min(reference.frames, current.frames);
  const channels = reference.channels;
  const fullScale = 32768;

  let maxAbsDelta = 0;
  const rmsByChannel = [];

  for (let c = 0; c < channels; c += 1) {
    let sumSq = 0;
    const r = reference.channelData[c];
    const cur = current.channelData[c];

    for (let i = 0; i < frameCount; i += 1) {
      const delta = r[i] - cur[i];
      const abs = Math.abs(delta);
      if (abs > maxAbsDelta) {
        maxAbsDelta = abs;
      }
      sumSq += delta * delta;
    }

    const rms = Math.sqrt(sumSq / Math.max(1, frameCount));
    rmsByChannel.push(rms / fullScale);
  }

  const rmsMax = Math.max(...rmsByChannel);
  return {
    maxAbsDelta,
    rmsByChannel,
    rmsMax,
    comparedFrames: frameCount,
  };
}

function main() {
  const [, , referenceDir, currentDir] = process.argv;
  if (!referenceDir || !currentDir) {
    usage();
    process.exit(2);
  }

  const referenceFiles = walkWavs(referenceDir);
  const currentFiles = walkWavs(currentDir);

  if (referenceFiles.length === 0) {
    console.error(`No reference WAV files found in: ${referenceDir}`);
    process.exit(2);
  }
  if (currentFiles.length === 0) {
    console.error(`No current WAV files found in: ${currentDir}`);
    process.exit(2);
  }

  const currentByRelativePath = new Map();
  for (const f of currentFiles) {
    const rel = path.relative(currentDir, f);
    currentByRelativePath.set(rel, f);
  }

  let failures = 0;
  let compared = 0;

  const rows = [];

  for (const refPath of referenceFiles) {
    const rel = path.relative(referenceDir, refPath);
    const curPath = currentByRelativePath.get(rel);

    if (!curPath) {
      rows.push({
        file: rel,
        status: 'FAIL',
        reason: 'missing current file',
      });
      failures += 1;
      continue;
    }

    let reference;
    let current;

    try {
      reference = readWav(refPath);
      current = readWav(curPath);
    } catch (err) {
      rows.push({
        file: rel,
        status: 'FAIL',
        reason: `parse error: ${err.message}`,
      });
      failures += 1;
      continue;
    }

    const formatMismatch =
      reference.sampleRate !== current.sampleRate ||
      reference.channels !== current.channels ||
      reference.bitsPerSample !== current.bitsPerSample;

    if (formatMismatch) {
      rows.push({
        file: rel,
        status: 'FAIL',
        reason: `format mismatch ref(${reference.sampleRate}Hz/${reference.channels}ch/${reference.bitsPerSample}bit) cur(${current.sampleRate}Hz/${current.channels}ch/${current.bitsPerSample}bit)`,
      });
      failures += 1;
      continue;
    }

    const durationMismatch = reference.frames !== current.frames;
    const refSilent = isSilent(reference.channelData);
    const curSilent = isSilent(current.channelData);
    const cmp = comparePcm(reference, current);

    const failReasons = [];
    if (durationMismatch) {
      failReasons.push(`duration mismatch (${reference.frames} vs ${current.frames} frames)`);
    }
    if (refSilent || curSilent) {
      failReasons.push(`silence detected (reference=${refSilent}, current=${curSilent})`);
    }
    if (cmp.rmsMax >= 0.01) {
      failReasons.push(`rms too high (${(cmp.rmsMax * 100).toFixed(3)}%)`);
    }

    if (failReasons.length > 0) {
      rows.push({
        file: rel,
        status: 'FAIL',
        reason: `${failReasons.join('; ')}; maxDelta=${cmp.maxAbsDelta}`,
      });
      failures += 1;
    } else {
      rows.push({
        file: rel,
        status: 'PASS',
        reason: `rms=${(cmp.rmsMax * 100).toFixed(3)}%, maxDelta=${cmp.maxAbsDelta}, frames=${cmp.comparedFrames}`,
      });
      compared += 1;
    }
  }

  const missingReferences = currentFiles
    .map((f) => path.relative(currentDir, f))
    .filter((rel) => !referenceFiles.some((rf) => path.relative(referenceDir, rf) === rel));

  for (const rel of missingReferences) {
    rows.push({
      file: rel,
      status: 'WARN',
      reason: 'current file has no matching reference',
    });
  }

  rows.sort((a, b) => a.file.localeCompare(b.file));

  console.log('Golden Master WAV Comparison');
  console.log(`Reference: ${referenceDir}`);
  console.log(`Current:   ${currentDir}`);
  console.log('');

  for (const row of rows) {
    console.log(`${row.status.padEnd(4)}  ${row.file}  ${row.reason}`);
  }

  console.log('');
  console.log(`Compared: ${compared}`);
  console.log(`Failures: ${failures}`);
  console.log(`Warnings: ${rows.filter((r) => r.status === 'WARN').length}`);

  process.exit(failures > 0 ? 1 : 0);
}

main();
