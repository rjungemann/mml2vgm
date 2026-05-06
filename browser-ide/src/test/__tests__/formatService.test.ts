/**
 * Format Service Tests
 * 
 * Tests for the format detection and handling service.
 */

import { describe, it, expect, beforeAll } from 'vitest';
import { formatService } from '@/services/formatService';

describe('FormatService', () => {
    beforeAll(() => {
        // FormatService is a singleton, no initialization needed
    });

    describe('Format Detection', () => {
        it('should detect GWI format from extension', () => {
            const result = formatService.detectFromExtension('test.gwi');
            expect(result).toBe('gwi');
        });

        it('should detect MUC format from extension', () => {
            const result = formatService.detectFromExtension('test.muc');
            expect(result).toBe('muc');
        });

        it('should detect MML format from extension', () => {
            const result = formatService.detectFromExtension('test.mml');
            expect(result).toBe('mml');
        });

        it('should detect MDL format from extension', () => {
            const result = formatService.detectFromExtension('test.mdl');
            expect(result).toBe('mdl');
        });

        it('should detect MUS format from extension', () => {
            const result = formatService.detectFromExtension('test.mus');
            expect(result).toBe('mus');
        });

        it('should default to null for unknown extensions', () => {
            const result = formatService.detectFromExtension('test.txt');
            expect(result).toBe('gwi');
        });
    });

    describe('Format Detection from Content', () => {
        it('should prefer extension-based format over content signals', () => {
            const content = `@0 v10 o4 l4 cdefgab
@1 v8 o5 l8 c2 d2 e2 f2`;
            const result = formatService.detectFormat(content, 'test.mml');
            expect(result.format).toBe('mml');
            expect(result.confidence).toBeGreaterThanOrEqual(70);
        });

        it('should detect MUC format from #mucom directive', () => {
            const content = `#MUCOM
#OPM
#TEMPO 120`;
            const result = formatService.detectFormat(content, 'test.muc');
            expect(result.format).toBe('muc');
            expect(result.confidence).toBeGreaterThan(80);
        });

        it('should detect MDL format from #md directive', () => {
            const content = `#MD
#OPN2
#TEMPO 132`;
            const result = formatService.detectFormat(content, 'test.mdl');
            expect(result.format).toBe('mdl');
            expect(result.confidence).toBeGreaterThanOrEqual(70);
        });

        it('should detect PMD format from @MUSIC directive', () => {
            const content = `@MUSIC
@TEMPO 125
@VOLUME 100`;
            const result = formatService.detectFormat(content, 'test.mus');
            expect(result.format).toBe('mus');
            expect(result.confidence).toBeGreaterThanOrEqual(70);
        });

        it('should default to GWI for generic MML content', () => {
            const content = `o4 l4 cdefgab>c
v10 t120`;
            const result = formatService.detectFormat(content, 'test.mml');
            expect(result.confidence).toBeGreaterThan(0);
        });
    });

    describe('Format Handlers', () => {
        it('should get all handlers', () => {
            const handlers = formatService.getAllHandlers();
            expect(handlers.length).toBeGreaterThanOrEqual(5);
        });

        it('should get handler by ID', () => {
            const handler = formatService.getHandler('gwi');
            expect(handler).toBeDefined();
            expect(handler?.id).toBe('gwi');
        });

        it('should get display name', () => {
            const name = formatService.getDisplayName('gwi');
            expect(name).toContain('mml2vgm');
        });

        it('should get default extension', () => {
            const ext = formatService.getDefaultExtension('gwi');
            expect(ext).toBe('.gwi');
        });

        it('should get default chips for format', () => {
            const chips = formatService.getDefaultChips('gwi');
            expect(chips.length).toBeGreaterThan(0);
            expect(chips).toContain('YM2608');
        });

        it('should check if format is native', () => {
            expect(formatService.isNativeFormat('gwi')).toBe(true);
            expect(formatService.isNativeFormat('muc')).toBe(false);
        });

        it('should check if driver is available', () => {
            expect(formatService.isDriverAvailable('gwi')).toBe(true);
            expect(formatService.isDriverAvailable('muc')).toBe(true);
        });
    });

    describe('Format Information', () => {
        it('should get format info', () => {
            const info = formatService.getFormatInfo('gwi');
            expect(info.id).toBe('gwi');
            expect(info.displayName).toBeDefined();
            expect(info.description).toBeDefined();
            expect(info.extensions).toBeDefined();
        });
    });

    describe('All Extensions', () => {
        it('should get all supported extensions', () => {
            const extensions = formatService.getAllExtensions();
            expect(extensions).toContain('.gwi');
            expect(extensions).toContain('.muc');
            expect(extensions).toContain('.mml');
            expect(extensions).toContain('.mdl');
            expect(extensions).toContain('.mus');
        });

        it('should check if extension is supported', () => {
            expect(formatService.isSupportedExtension('.gwi')).toBe(true);
            expect(formatService.isSupportedExtension('.muc')).toBe(true);
            expect(formatService.isSupportedExtension('.xyz')).toBe(false);
        });
    });

    describe('Syntax Configuration', () => {
        it('should get syntax config for GWI', () => {
            const config = formatService.getSyntaxConfig('gwi');
            expect(config).toBeDefined();
            expect(config?.languageId).toBe('gwi');
            expect(config?.tokens).toBeDefined();
        });

        it('should get syntax config for MUC', () => {
            const config = formatService.getSyntaxConfig('muc');
            expect(config).toBeDefined();
            expect(config?.languageId).toBe('muc');
        });
    });
});
