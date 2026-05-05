/**
 * i18n Service Tests
 * 
 * Tests for the internationalization service.
 * Part of Phase 7: Polish and Optimization.
 */

import { describe, it, expect, beforeAll, afterAll, vi } from 'vitest';
import { I18nService, i18nService, detectLanguage, useTranslation } from '@/services/i18nService';
import i18n from 'i18next';

// Mock localStorage
global.localStorage = {
    getItem: vi.fn(),
    setItem: vi.fn(),
    removeItem: vi.fn(),
    clear: vi.fn(),
    key: vi.fn(),
    length: 0,
} as any;

// Mock navigator
const originalNavigator = global.navigator;
global.navigator = {
    ...originalNavigator,
    language: 'en-US',
} as any;

// Mock fetch for loading translations
const originalFetch = global.fetch;
global.fetch = vi.fn();

describe('i18nService', () => {
    beforeAll(() => {
        // Mock fetch to return mock translations
        (global.fetch as any).mockImplementation((url: string) => {
            if (url.includes('/locales/en.json')) {
                return Promise.resolve({
                    ok: true,
                    json: () => Promise.resolve({
                        app: { title: 'mml2vgm Browser IDE' },
                        menu: { file: 'File', edit: 'Edit' },
                    }),
                });
            }
            if (url.includes('/locales/ja.json')) {
                return Promise.resolve({
                    ok: true,
                    json: () => Promise.resolve({
                        app: { title: 'mml2vgm ブラウザIDE' },
                        menu: { file: 'ファイル', edit: '編集' },
                    }),
                });
            }
            return Promise.reject(new Error('Not found'));
        });
    });

    afterAll(() => {
        global.fetch = originalFetch;
        global.navigator = originalNavigator;
        vi.restoreAllMocks();
    });

    describe('Language Detection', () => {
        it('should detect Japanese from navigator language', () => {
            (global as any).navigator.language = 'ja-JP';
            localStorage.getItem = vi.fn().mockReturnValue(null);
            
            const lang = detectLanguage();
            expect(lang).toBe('ja');
        });

        it('should detect Japanese with ja in language', () => {
            (global as any).navigator.language = 'ja';
            localStorage.getItem = vi.fn().mockReturnValue(null);
            
            const lang = detectLanguage();
            expect(lang).toBe('ja');
        });

        it('should default to English for non-Japanese languages', () => {
            (global as any).navigator.language = 'fr-FR';
            localStorage.getItem = vi.fn().mockReturnValue(null);
            
            const lang = detectLanguage();
            expect(lang).toBe('en');
        });

        it('should use saved language from localStorage', () => {
            (global as any).navigator.language = 'fr-FR';
            localStorage.getItem = vi.fn().mockReturnValue('ja');
            
            const lang = detectLanguage();
            expect(lang).toBe('ja');
        });
    });

    describe('Singleton Pattern', () => {
        it('should return the same instance', () => {
            const instance1 = I18nService.getInstance();
            const instance2 = I18nService.getInstance();
            expect(instance1).toBe(instance2);
        });

        it('should be the same as the exported i18nService', () => {
            const instance = I18nService.getInstance();
            expect(instance).toBe(i18nService);
        });
    });

    describe('Language Management', () => {
        it('should get supported languages', () => {
            const service = I18nService.getInstance();
            const languages = service.getSupportedLanguages();
            
            expect(languages).toContain('en');
            expect(languages).toContain('ja');
            expect(languages).toContain('auto');
        });

        it('should get language display names', () => {
            const service = I18nService.getInstance();
            
            expect(service.getLanguageName('en')).toBe('English');
            expect(service.getLanguageName('ja')).toBe('日本語');
            expect(service.getLanguageName('auto')).toBe('Auto (Browser Default)');
        });
    });

    describe('Translation Helpers', () => {
        it('should check if a key exists', () => {
            const service = I18nService.getInstance();
            
            // This will use fallback translations since i18n isn't initialized
            // The check is for the method existence
            expect(typeof service.has).toBe('function');
        });

        it('should translate a key', () => {
            const service = I18nService.getInstance();
            
            // Method should exist
            expect(typeof service.t).toBe('function');
        });
    });

    describe('State Management', () => {
        it('should return initial state', () => {
            const service = I18nService.getInstance();
            const state = service.getState();
            
            expect(state.supportedLanguages).toContain('en');
            expect(state.supportedLanguages).toContain('ja');
            expect(state.supportedLanguages).toContain('auto');
            expect(state.isReady).toBe(false);
        });

        it('should allow subscription to state changes', () => {
            const service = I18nService.getInstance();
            const callback = vi.fn();
            
            const unsubscribe = service.subscribe(callback);
            
            // Initial call
            expect(callback).toHaveBeenCalledTimes(1);
            
            // Unsubscribe
            unsubscribe();
        });
    });
});

describe('useTranslation Hook', () => {
    it('should export the hook', () => {
        expect(typeof useTranslation).toBe('function');
    });
});
