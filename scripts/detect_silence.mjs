#!/usr/bin/env node
// Reads one or more WAV files (or directories of WAV files) and reports
// which files contain only silence (all samples == 0).
// Exit code 0 = all files have audio; 1 = at least one silent file.

import fs from 'node:fs';
import path from 'node:path';

function usage() {
  console.error('Usage: node scripts/detect_silence.mjs <path> [<path> ...]');
  console.error('  Each path may be a .wav file or a directory (searched recursively).');
}

function walkWavs(dir) {
  const out = [];
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

function collectPaths(args) {
  const files = [];
  for (const arg of args) {
    const stat = fs.statSync(arg);
    if (stat.isDirectory()) {
      files.push(...walkWavs(arg));
    } else if (arg.toLowerCase().endsWith('.wav')) {
      files.push(arg);
    } else {
      console.error(`Skipping non-WAV path: ${arg}`);
    }
  }
  return files;
}

function readWav(filePath) {
  const buf = fs.readFileSync(filePath);

  if (buf.length < 44) throw new Error('file too short to be a valid WAV');
  if (buf.toString('ascii', 0, 4) !== 'RIFF' || buf.toString('ascii', 8, 12) !== 'WAVE') {
    throw new Error('missing RIFF/WAVE header');
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
      throw new Error(`corrupt chunk "${chunkId}"`);
    }

    if (chunkId === 'fmt ') {
      if (chunkSize < 16) throw new Error(`unsupported fmt chunk size ${chunkSize}`);
      const audioFormat = buf.readUInt16LE(chunkDataStart);
      fmt = {
        audioFormat,
        channels: buf.readUInt16LE(chunkDataStart + 2),
        bitsPerSample: buf.readUInt16LE(chunkDataStart + 14),
        // WAVE_FORMAT_EXTENSIBLE: SubFormat GUID first two bytes give the actual codec tag.
        subFormatTag: audioFormat === 0xfffe && chunkSize >= 40
          ? buf.readUInt16LE(chunkDataStart + 24)
          : null,
      };
    } else if (chunkId === 'data') {
      dataOffset = chunkDataStart;
      dataSize = chunkSize;
      break;
    }

    offset = chunkDataStart + chunkSize + (chunkSize % 2);
  }

  if (!fmt) throw new Error('missing fmt chunk');
  if (dataOffset < 0) throw new Error('missing data chunk');

  // 0xFFFE = WAVE_FORMAT_EXTENSIBLE; check SubFormat GUID for PCM (0x0001).
  if (fmt.audioFormat === 0xfffe) {
    if (fmt.subFormatTag !== 1) {
      throw new Error(`extensible WAV with non-PCM subformat ${fmt.subFormatTag}`);
    }
  } else if (fmt.audioFormat !== 1) {
    throw new Error(`unsupported audio format ${fmt.audioFormat} (expected PCM=1)`);
  }

  if (fmt.bitsPerSample !== 16) throw new Error(`unsupported bit depth ${fmt.bitsPerSample} (expected 16)`);

  const bytesPerSample = fmt.bitsPerSample / 8;
  const frameSize = bytesPerSample * fmt.channels;
  const frames = Math.floor(dataSize / frameSize);
  const channelData = Array.from({ length: fmt.channels }, () => new Int16Array(frames));

  let pos = dataOffset;
  for (let i = 0; i < frames; i += 1) {
    for (let c = 0; c < fmt.channels; c += 1) {
      channelData[c][i] = buf.readInt16LE(pos);
      pos += 2;
    }
  }

  return { channels: fmt.channels, frames, channelData };
}

function isSilent(channelData) {
  for (const channel of channelData) {
    for (let i = 0; i < channel.length; i += 1) {
      if (channel[i] !== 0) return false;
    }
  }
  return true;
}

function peakAmplitude(channelData) {
  let peak = 0;
  for (const channel of channelData) {
    for (let i = 0; i < channel.length; i += 1) {
      const abs = Math.abs(channel[i]);
      if (abs > peak) peak = abs;
    }
  }
  return peak;
}

function main() {
  const args = process.argv.slice(2);
  if (args.length === 0) {
    usage();
    process.exit(2);
  }

  const files = collectPaths(args);
  if (files.length === 0) {
    console.error('No WAV files found.');
    process.exit(2);
  }

  let silentCount = 0;
  let errorCount = 0;

  for (const filePath of files) {
    let wav;
    try {
      wav = readWav(filePath);
    } catch (err) {
      console.log(`ERROR  ${filePath}  ${err.message}`);
      errorCount += 1;
      continue;
    }

    const silent = isSilent(wav.channelData);
    const peak = peakAmplitude(wav.channelData);
    const label = silent ? 'SILENT' : 'OK    ';
    console.log(`${label}  ${filePath}  frames=${wav.frames} peak=${peak}`);
    if (silent) silentCount += 1;
  }

  console.log('');
  console.log(`Checked: ${files.length}  Silent: ${silentCount}  Errors: ${errorCount}`);

  process.exit(silentCount > 0 || errorCount > 0 ? 1 : 0);
}

main();
