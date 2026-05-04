/**
 * Part Service
 * 
 * Manages parsing and management of MML parts/channels.
 * Extracts part information from MML source code and provides
 * part management functionality.
 */

import type { PartInfo, SoundChip, Position } from '@/types';

// ============================================================================
// Types
// ============================================================================

/** Part definition from MML source */
export interface MmlPartDefinition {
    index: number;
    name: string;
    chip: SoundChip;
    channel: number;
    startPosition: Position;
    endPosition: Position;
}

/** Part with runtime state */
export interface PartWithState extends PartInfo {
    startPosition: Position;
    endPosition: Position;
}

// ============================================================================
// Part Service Class
// ============================================================================

/**
 * Service for parsing and managing MML parts.
 * 
 * This class provides:
 * - Parsing parts from MML source code
 * - Part management (mute/solo/volume/pan)
 * - Part information for display
 */
export class PartService {
    private static instance: PartService | null = null;
    
    // Parsed parts from current document
    private _parts: PartWithState[] = [];
    
    // Part state overrides (mute/solo/volume/pan)
    private _partOverrides: Map<number, Partial<PartInfo>> = new Map();
    
    // Event listeners for part changes
    private _listeners: Array<(parts: PartWithState[]) => void> = [];
    
    // ========================================================================
    // Singleton
    // ========================================================================
    
    public static getInstance(): PartService {
        if (!PartService.instance) {
            PartService.instance = new PartService();
        }
        return PartService.instance;
    }
    
    private constructor() {
        // Private constructor for singleton
    }
    
    // ========================================================================
    // Parsing
    // ========================================================================
    
    /**
     * Parse parts from MML source code.
     * 
     * @param mml - The MML source code
     * @param documentId - Optional document ID for tracking
     * @returns Array of parsed part definitions
     */
    public parseFromMML(mml: string, _documentId: string | null = null): PartWithState[] {
        this._parts = [];
        this._partOverrides.clear();
        
        const parts: PartWithState[] = [];
        const lines = mml.split('\n');
        
        // Default chip for parts
        let currentChip: SoundChip = 'YM2608';
        let currentChannel = 0;
        
        // Parse each line for part definitions
        for (let lineNum = 0; lineNum < lines.length; lineNum++) {
            const line = lines[lineNum];
            const trimmedLine = line.trim();
            
            // Skip empty lines and comments
            if (!trimmedLine || trimmedLine.startsWith(';') || trimmedLine.startsWith('//')) {
                continue;
            }
            
            // Check for chip definition (e.g., #CHIP YM2608)
            // This is a simplification - actual MML may use different syntax
            
            // Check for part definition (e.g., @0, @1, @2, etc.)
            const partMatch = trimmedLine.match(/^@(\d+)/);
            if (partMatch) {
                const partIndex = parseInt(partMatch[1], 10);
                
                // Parse part name if present after the index
                // Format: @0 FM1 or @0 "FM1" or @0
                const restOfLine = trimmedLine.substring(partMatch[0].length).trim();
                let partName = `Part${partIndex}`;
                
                if (restOfLine) {
                    // Try to extract name from quotes or as first word
                    const nameMatch = restOfLine.match(/^"([^"]+)"/) || 
                                     restOfLine.match(/^\S+/);
                    if (nameMatch) {
                        partName = nameMatch[1] || nameMatch[0];
                    }
                }
                
                // Create part definition
                const part: PartWithState = {
                    index: partIndex,
                    name: partName,
                    chip: currentChip,
                    channel: currentChannel,
                    volume: 100,
                    pan: 0,
                    isSolo: false,
                    isMuted: false,
                    isKbdAssigned: false,
                    startPosition: { line: lineNum + 1, column: trimmedLine.indexOf(partMatch[0]) + 1 },
                    endPosition: { line: lineNum + 1, column: trimmedLine.length },
                };
                
                parts.push(part);
                currentChannel = partIndex % 6; // Default: up to 6 channels per chip
            }
            
            // Check for chip-specific commands that might indicate part/chip assignment
            // This is a simplification - actual parsing would be more complex
        }
        
        // If no parts found, create default parts based on partCount
        // This handles cases where parts are implicit
        if (parts.length === 0) {
            // Create default 6 parts (3 FM + 3 SSG for YM2608)
            for (let i = 0; i < 6; i++) {
                const part: PartWithState = {
                    index: i,
                    name: i < 3 ? `FM${i + 1}` : `SSG${i - 2}`,
                    chip: i < 3 ? 'YM2608' : 'AY8910',
                    channel: i % 3,
                    volume: 100,
                    pan: 0,
                    isSolo: false,
                    isMuted: false,
                    isKbdAssigned: false,
                    startPosition: { line: 1, column: 1 },
                    endPosition: { line: 1, column: 1 },
                };
                parts.push(part);
            }
        }
        
        // Sort parts by index
        parts.sort((a, b) => a.index - b.index);
        
        this._parts = parts;
        this.emitUpdate();
        
        return parts;
    }
    
    /**
     * Parse parts from compile result metadata.
     * Uses partCount to create default parts if real part info not available.
     * 
     * @param partCount - Number of parts from compilation
     * @param chipsUsed - Array of chips used
     * @param documentId - Optional document ID for tracking
     */
    public parseFromCompileResult(
        partCount: number,
        chipsUsed: string[],
        _documentId: string | null = null
    ): PartWithState[] {
        this._parts = [];
        this._partOverrides.clear();
        
        const parts: PartWithState[] = [];
        
        // If we have real part info from compile, use it
        // For now, create default parts based on partCount
        
        // Determine chip assignments
        const primaryChip: SoundChip = (chipsUsed[0] as SoundChip) || 'YM2608';
        const secondaryChip: SoundChip | null = chipsUsed.length > 1 
            ? (chipsUsed[1] as SoundChip) 
            : null;
        
        for (let i = 0; i < partCount; i++) {
            // Assign to primary or secondary chip
            const chip = i < 6 ? primaryChip : (secondaryChip || primaryChip);
            const channel = i % (chip === 'AY8910' || chip === 'SN76489' ? 3 : 6);
            
            const part: PartWithState = {
                index: i,
                name: this.generatePartName(i, chip),
                chip: chip,
                channel: channel,
                volume: 100,
                pan: 0,
                isSolo: false,
                isMuted: false,
                isKbdAssigned: false,
                startPosition: { line: 1, column: 1 },
                endPosition: { line: 1, column: 1 },
            };
            parts.push(part);
        }
        
        // If no parts and we have chips, create default parts
        if (parts.length === 0 && chipsUsed.length > 0) {
            const chip = chipsUsed[0] as SoundChip;
            const channelCount = this.getChannelCount(chip);
            for (let i = 0; i < channelCount; i++) {
                const part: PartWithState = {
                    index: i,
                    name: this.generatePartName(i, chip),
                    chip: chip,
                    channel: i,
                    volume: 100,
                    pan: 0,
                    isSolo: false,
                    isMuted: false,
                    isKbdAssigned: false,
                    startPosition: { line: 1, column: 1 },
                    endPosition: { line: 1, column: 1 },
                };
                parts.push(part);
            }
        }
        
        // Sort parts by index
        parts.sort((a, b) => a.index - b.index);
        
        this._parts = parts;
        this.emitUpdate();
        
        return parts;
    }
    
    /**
     * Get channel count for a chip.
     */
    private getChannelCount(chip: SoundChip): number {
        // FM chips typically have 6 channels, PSG chips have 3-4
        if (chip.includes('YM26') || chip.includes('YM2151') || chip.includes('YM3526') || 
            chip.includes('Y8950') || chip.includes('YM3812') || chip.includes('YMF262')) {
            return 6; // FM operators
        }
        if (chip.includes('SN76489') || chip.includes('AY8910') || chip.includes('K05')) {
            return 3; // PSG channels
        }
        if (chip.includes('RF5C164') || chip.includes('SegaPCM')) {
            return 8; // PCM channels
        }
        return 6; // Default
    }
    
    /**
     * Generate a default name for a part based on index and chip.
     */
    private generatePartName(index: number, chip: SoundChip): string {
        if (chip.includes('YM26') || chip.includes('OPN') || chip.includes('YM2151')) {
            return `FM${index + 1}`;
        }
        if (chip.includes('SN76489') || chip.includes('AY8910') || chip.includes('PSG')) {
            return `SSG${index + 1}`;
        }
        if (chip.includes('RF5C164') || chip.includes('PCM')) {
            return `PCM${index + 1}`;
        }
        return `Part${index + 1}`;
    }
    
    // ========================================================================
    // Accessors
    // ========================================================================
    
    /**
     * Get all parts.
     */
    public getParts(): PartWithState[] {
        // Return parts with overrides applied
        return this._parts.map(part => ({
            ...part,
            ...this._partOverrides.get(part.index),
        }));
    }
    
    /**
     * Get part by index.
     */
    public getPart(index: number): PartWithState | null {
        const part = this._parts.find(p => p.index === index);
        if (!part) return null;
        
        return {
            ...part,
            ...this._partOverrides.get(index),
        };
    }
    
    /**
     * Get part count.
     */
    public getPartCount(): number {
        return this._parts.length;
    }
    
    /**
     * Get parts for a specific chip.
     */
    public getPartsForChip(chip: SoundChip): PartWithState[] {
        return this.getParts().filter(p => p.chip === chip);
    }
    
    /**
     * Get parts assigned to a specific channel.
     */
    public getPartsForChannel(channel: number): PartWithState[] {
        return this.getParts().filter(p => p.channel === channel);
    }
    
    // ========================================================================
    // Part Management
    // ========================================================================
    
    /**
     * Update part properties.
     */
    public updatePart(index: number, updates: Partial<PartInfo>): void {
        const part = this._parts.find(p => p.index === index);
        if (part) {
            this._partOverrides.set(index, { ...this._partOverrides.get(index), ...updates });
            this.emitUpdate();
        }
    }
    
    /**
     * Toggle mute for a part.
     */
    public toggleMute(index: number): void {
        const part = this.getPart(index);
        if (part) {
            this.updatePart(index, { isMuted: !part.isMuted });
        }
    }
    
    /**
     * Toggle solo for a part.
     */
    public toggleSolo(index: number): void {
        // If solo is being enabled, clear other solos first
        if (!this._parts.find(p => p.index === index)?.isSolo) {
            this._parts.forEach(p => {
                this._partOverrides.set(p.index, { ...this._partOverrides.get(p.index), isSolo: false });
            });
        }
        
        const part = this.getPart(index);
        if (part) {
            this.updatePart(index, { isSolo: !part.isSolo });
        }
    }
    
    /**
     * Set volume for a part.
     */
    public setVolume(index: number, volume: number): void {
        this.updatePart(index, { volume: Math.max(0, Math.min(100, volume)) });
    }
    
    /**
     * Set pan for a part.
     */
    public setPan(index: number, pan: number): void {
        this.updatePart(index, { pan: Math.max(-100, Math.min(100, pan)) });
    }
    
    /**
     * Assign keyboard to a part.
     */
    public assignKbd(index: number): void {
        // Clear any existing KBD assignment
        this._parts.forEach(p => {
            this._partOverrides.set(p.index, { ...this._partOverrides.get(p.index), isKbdAssigned: false });
        });
        this.updatePart(index, { isKbdAssigned: true });
    }
    
    /**
     * Clear keyboard assignment.
     */
    public clearKbdAssignment(): void {
        this._parts.forEach(p => {
            this._partOverrides.set(p.index, { ...this._partOverrides.get(p.index), isKbdAssigned: false });
        });
        this.emitUpdate();
    }
    
    /**
     * Reset all part overrides.
     */
    public resetAll(): void {
        this._partOverrides.clear();
        this.emitUpdate();
    }
    
    // ========================================================================
    // Active Parts (for trace playback)
    // ========================================================================
    
    /**
     * Set active parts during playback.
     */
    public setActiveParts(_partIndices: number[]): void {
        // This could be used to update the UI during trace playback
        // For now, just emit an update
        this.emitUpdate();
    }
    
    /**
     * Get parts that should be highlighted as active.
     * This is used during trace playback.
     */
    public getActiveParts(): Set<string> {
        // This would be connected to traceService
        // For now, return empty set
        return new Set();
    }
    
    // ========================================================================
    // Event Listeners
    // ========================================================================
    
    /**
     * Add a listener for part changes.
     */
    public addListener(listener: (parts: PartWithState[]) => void): void {
        this._listeners.push(listener);
    }
    
    /**
     * Remove a listener.
     */
    public removeListener(listener: (parts: PartWithState[]) => void): void {
        this._listeners = this._listeners.filter(l => l !== listener);
    }
    
    /**
     * Emit update to all listeners.
     */
    private emitUpdate(): void {
        this._listeners.forEach(listener => listener(this.getParts()));
    }
    
    // ========================================================================
    // Utility Methods
    // ========================================================================
    
    /**
     * Get all solo parts.
     */
    public getSoloParts(): PartWithState[] {
        return this.getParts().filter(p => p.isSolo);
    }
    
    /**
     * Get all muted parts.
     */
    public getMutedParts(): PartWithState[] {
        return this.getParts().filter(p => p.isMuted);
    }
    
    /**
     * Get the KBD assigned part.
     */
    public getKbdAssignedPart(): PartWithState | null {
        return this.getParts().find(p => p.isKbdAssigned) || null;
    }
    
    /**
     * Check if any part is solo.
     */
    public hasSolo(): boolean {
        return this.getParts().some(p => p.isSolo);
    }
    
    /**
     * Get effective volume for a part considering solo/mute state.
     */
    public getEffectiveVolume(index: number): number {
        const part = this.getPart(index);
        if (!part) return 0;
        
        // If solo is enabled and this part is not solo, volume is 0
        if (this.hasSolo() && !part.isSolo) {
            return 0;
        }
        
        // If muted, volume is 0
        if (part.isMuted) {
            return 0;
        }
        
        return part.volume;
    }
}

// ============================================================================
// Singleton Export
// ============================================================================

export const partService = PartService.getInstance();
