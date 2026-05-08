/**
 * Instrument Parser and Serializer
 *
 * Round-trip helpers for all MML instrument definition types:
 *   FM Tone  ('@ M NNN  or  '@ F NNN)
 *   PCM      ('@ P NNN, "file", freq, vol, Chip)
 *   Envelope ('@ E NNN, v0,v1,v2,...)
 *   Arpeggio ('@ A NNN, note,note,...)
 *
 * The MML source text is the single source of truth; these helpers
 * parse it into data structures, let the UI mutate them, then write
 * the updated block back using replaceDefinitionBlock().
 */

// ============================================================================
// FM Instrument
// ============================================================================

/** One operator row (11 parameters). Index order: AR DR SR RR SL TL KS ML DT AM SSG */
export type FmOperator = [number, number, number, number, number, number, number, number, number, number, number];

export const FM_PARAM_NAMES = ['AR', 'DR', 'SR', 'RR', 'SL', 'TL', 'KS', 'ML', 'DT', 'AM', 'SSG'] as const;
export const FM_PARAM_MAX  = [  31,   31,   31,   15,   15,  127,    3,   15,    7,    1,   15] as const;

export type FmType = 'M' | 'F';

export interface FmInstrumentDef {
    /** 0-based instrument number */
    number: number;
    /** M-type (auto-TL) or F-type (explicit carrier TL) */
    type: FmType;
    /** Optional patch name from header line */
    name: string;
    /** 4 operator rows in MML order (OP1..OP4) */
    ops: [FmOperator, FmOperator, FmOperator, FmOperator];
    /** Algorithm 0-7 */
    alg: number;
    /** Feedback 0-7 */
    fb: number;
    /** Line index of the opening "'@ M/F NNN" line in the source */
    startLine: number;
    /** Line index of the final ALG/FB row */
    endLine: number;
}

const DEFAULT_OP: FmOperator = [31, 0, 0, 7, 0, 0, 0, 1, 0, 0, 0];

export function defaultFmInstrument(number: number): FmInstrumentDef {
    return {
        number,
        type: 'M',
        name: '',
        ops: [
            [...DEFAULT_OP] as FmOperator,
            [...DEFAULT_OP] as FmOperator,
            [...DEFAULT_OP] as FmOperator,
            [...DEFAULT_OP] as FmOperator,
        ],
        alg: 7,
        fb: 0,
        startLine: -1,
        endLine: -1,
    };
}

/** Parse all FM instrument definitions from a source string. */
export function parseFmInstruments(source: string): FmInstrumentDef[] {
    const lines = source.split('\n');
    const results: FmInstrumentDef[] = [];

    let i = 0;
    while (i < lines.length) {
        const line = lines[i].trim();
        // Match: '@ M 000 or '@ F 001 "optional name"
        const header = line.match(/^'@\s+([MF])\s+(\d+)\s*(?:"([^"]*)")?/i);
        if (header) {
            const type = header[1].toUpperCase() as FmType;
            const number = parseInt(header[2], 10);
            const name = header[3] ?? '';
            const startLine = i;

            // Skip comment/header line if present (the one with AR DR SR ...)
            let j = i + 1;
            if (j < lines.length && /^\s*(AR|;)/.test(lines[j])) j++;

            // Read 4 operator rows
            const ops: FmOperator[] = [];
            while (j < lines.length && ops.length < 4) {
                const opLine = lines[j].trim();
                const opMatch = opLine.match(/^'@\s+([\d,]+)/);
                if (opMatch) {
                    const vals = opMatch[1].split(',').map(Number);
                    // Pad to 11 parameters
                    while (vals.length < 11) vals.push(0);
                    ops.push(vals.slice(0, 11) as FmOperator);
                    j++;
                } else if (opLine === '' || opLine.startsWith(';')) {
                    j++;
                } else {
                    break;
                }
            }

            if (ops.length !== 4) { i++; continue; }

            // Read ALG/FB row
            let alg = 7, fb = 0, endLine = j;
            while (j < lines.length) {
                const algLine = lines[j].trim();
                const algMatch = algLine.match(/^'@\s+(\d+),(\d+)/);
                if (algMatch) {
                    alg = parseInt(algMatch[1], 10);
                    fb  = parseInt(algMatch[2], 10);
                    endLine = j;
                    j++;
                    break;
                } else if (algLine === '' || algLine.startsWith(';')) {
                    j++;
                } else {
                    break;
                }
            }

            results.push({
                number,
                type,
                name,
                ops: ops as [FmOperator, FmOperator, FmOperator, FmOperator],
                alg,
                fb,
                startLine,
                endLine,
            });

            i = j;
        } else {
            i++;
        }
    }

    return results;
}

/** Serialize a single FM instrument definition back to MML text. */
export function serializeFmInstrument(inst: FmInstrumentDef): string {
    const num = String(inst.number).padStart(3, '0');
    const namePart = inst.name ? ` "${inst.name}"` : '';
    const header = `'@ ${inst.type} ${num}${namePart}`;
    const colHeader = `   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG`;
    const opLines = inst.ops.map((op) =>
        `'@ ${op.map((v) => String(v).padStart(3, '0')).join(',')}`
    );
    const algLine = `   ALG FB`;
    const algFbLine = `'@ ${String(inst.alg).padStart(3, '0')},${String(inst.fb).padStart(3, '0')}`;

    return [header, colHeader, ...opLines, algLine, algFbLine].join('\n');
}

// ============================================================================
// PCM Instrument
// ============================================================================

export interface PcmInstrumentDef {
    number: number;
    filename: string;
    frequency: number;
    volume: number;
    chip: string;
    option: string;
    startLine: number;
    endLine: number;
}

export function parsePcmInstruments(source: string): PcmInstrumentDef[] {
    const lines = source.split('\n');
    const results: PcmInstrumentDef[] = [];

    lines.forEach((line, i) => {
        // '@ P 001,"kick.wav",8000,100,YM2612
        const m = line.trim().match(/^'@\s+P\s+(\d+)\s*,\s*"([^"]*)"\s*,\s*(\d+)\s*,\s*(\d+)\s*,\s*(\w+)\s*(?:,\s*(\d+))?/i);
        if (m) {
            results.push({
                number: parseInt(m[1], 10),
                filename: m[2],
                frequency: parseInt(m[3], 10),
                volume: parseInt(m[4], 10),
                chip: m[5],
                option: m[6] ?? '',
                startLine: i,
                endLine: i,
            });
        }
    });

    return results;
}

export function serializePcmInstrument(inst: PcmInstrumentDef): string {
    const num = String(inst.number).padStart(3, '0');
    const opt = inst.option ? `,${inst.option}` : '';
    return `'@ P ${num},"${inst.filename}",${inst.frequency},${inst.volume},${inst.chip}${opt}`;
}

// ============================================================================
// Envelope ('@ E NNN, v0,v1,v2,...)
// ============================================================================

export interface EnvelopeDef {
    number: number;
    steps: number[];
    startLine: number;
    endLine: number;
}

export function parseEnvelopes(source: string): EnvelopeDef[] {
    const lines = source.split('\n');
    const results: EnvelopeDef[] = [];

    lines.forEach((line, i) => {
        const m = line.trim().match(/^'@\s+E\s+(\d+)\s*,?\s*([\d,\s]*)/i);
        if (m) {
            const steps = m[2]
                .split(',')
                .map((s) => parseInt(s.trim(), 10))
                .filter((v) => !isNaN(v));
            results.push({ number: parseInt(m[1], 10), steps, startLine: i, endLine: i });
        }
    });

    return results;
}

export function serializeEnvelope(env: EnvelopeDef): string {
    const num = String(env.number).padStart(3, '0');
    return `'@ E ${num}, ${env.steps.join(',')}`;
}

// ============================================================================
// Arpeggio ('@ A NNN, note,note,...)
// ============================================================================

export interface ArpeggioDef {
    number: number;
    notes: string[];
    startLine: number;
    endLine: number;
}

export function parseArpeggios(source: string): ArpeggioDef[] {
    const lines = source.split('\n');
    const results: ArpeggioDef[] = [];

    lines.forEach((line, i) => {
        const m = line.trim().match(/^'@\s+A\s+(\d+)\s*,?\s*(.*)/i);
        if (m) {
            const notes = m[2]
                .split(',')
                .map((s) => s.trim())
                .filter(Boolean);
            results.push({ number: parseInt(m[1], 10), notes, startLine: i, endLine: i });
        }
    });

    return results;
}

export function serializeArpeggio(arp: ArpeggioDef): string {
    const num = String(arp.number).padStart(3, '0');
    return `'@ A ${num}, ${arp.notes.join(',')}`;
}

// ============================================================================
// replaceDefinitionBlock
// ============================================================================

/**
 * Replace an existing instrument definition block in source text with a new one.
 * If startLine === -1 (new instrument), append to end of source.
 */
export function replaceDefinitionBlock(
    source: string,
    startLine: number,
    endLine: number,
    newBlock: string,
): string {
    if (startLine === -1) {
        // New instrument — append after a blank line
        const sep = source.endsWith('\n') ? '\n' : '\n\n';
        return source + sep + newBlock + '\n';
    }

    const lines = source.split('\n');
    const before = lines.slice(0, startLine);
    const after = lines.slice(endLine + 1);
    return [...before, newBlock, ...after].join('\n');
}

// ============================================================================
// Algorithm helpers
// ============================================================================

/** Returns which OP indices (0-based) are carriers for a given algorithm. */
export function getCarrierOps(alg: number): number[] {
    switch (alg) {
        case 0: return [3];
        case 1: return [3];
        case 2: return [3];
        case 3: return [3];
        case 4: return [2, 3];
        case 5: return [1, 2, 3];
        case 6: return [1, 2, 3];
        case 7: return [0, 1, 2, 3];
        default: return [3];
    }
}
