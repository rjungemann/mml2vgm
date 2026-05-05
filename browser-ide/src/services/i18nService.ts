/**
 * i18n Service
 * 
 * Provides internationalization support for the browser IDE.
 * Supports English and Japanese languages.
 * 
 * This is part of Phase 7: Polish and Optimization.
 */

import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

// ============================================================================
// Types
// ============================================================================

/** Supported languages */
export type SupportedLanguage = 'en' | 'ja' | 'auto';

/** i18n service state */
export interface I18nServiceState {
    language: SupportedLanguage;
    isReady: boolean;
    error: string | null;
    supportedLanguages: SupportedLanguage[];
}

// ============================================================================
// Language Detection
// ============================================================================

/**
 * Detect user's preferred language from browser settings.
 */
export function detectLanguage(): SupportedLanguage {
    // Check localStorage first
    const savedLanguage = localStorage.getItem('mml2vgm-language');
    if (savedLanguage && (savedLanguage === 'en' || savedLanguage === 'ja')) {
        return savedLanguage as SupportedLanguage;
    }
    
    // Check navigator language
    const navigatorLang = navigator.language.toLowerCase();
    
    // Check if Japanese
    if (navigatorLang.startsWith('ja') || navigatorLang.includes('ja')) {
        return 'ja';
    }
    
    // Default to English
    return 'en';
}

// ============================================================================
// i18n Configuration
// ============================================================================

const i18nConfig = {
    // Default language
    lng: detectLanguage(),
    
    // Fallback language
    fallbackLng: 'en',
    
    // Supported languages
    supportedLngs: ['en', 'ja'],
    
    // Disable escape for HTML
    interpolation: {
        escapeValue: false,
    },
    
    // Debug mode in development
    debug: import.meta.env.DEV,
    
    // React specific configuration
    react: {
        useSuspense: false,
    },
};

// ============================================================================
// i18n Service Class
// ============================================================================

/**
 * i18n Service
 * 
 * Manages internationalization for the browser IDE.
 * Provides language switching and translation loading.
 */
export class I18nService {
    private static instance: I18nService | null = null;
    
    private _language: SupportedLanguage;
    private _isReady: boolean = false;
    private _error: string | null = null;
    private _initPromise: Promise<void> | null = null;
    
    // Event listeners
    private stateListeners: Array<(state: I18nServiceState) => void> = [];
    
    // ========================================================================
    // Singleton
    // ========================================================================
    
    public static getInstance(): I18nService {
        if (!I18nService.instance) {
            I18nService.instance = new I18nService();
        }
        return I18nService.instance;
    }
    
    private constructor() {
        this._language = detectLanguage();
    }
    
    // ========================================================================
    // Initialization
    // ========================================================================
    
    /**
     * Initialize i18n.
     */
    public async init(): Promise<void> {
        if (this._isReady) {
            return;
        }

        if (this._initPromise) {
            return this._initPromise;
        }

        if (i18n.isInitialized) {
            this._isReady = true;
            this._error = null;
            this.notifyListeners();
            return;
        }

        this._initPromise = (async () => {
        try {
            // Initialize i18next
            await i18n.use(initReactI18next).init({
                ...i18nConfig,
                lng: this._language === 'auto' ? detectLanguage() : this._language,
            });
            
            // Load translations
            await this.loadTranslations();
            
            this._isReady = true;
            this._error = null;
            this.notifyListeners();
            
            console.log(`[i18n] Initialized with language: ${this._language}`);
        } catch (error) {
            this._error = `Failed to initialize i18n: ${error}`;
            this.notifyListeners();
            console.error('[i18n] Initialization error:', error);
        }
        })();

        try {
            await this._initPromise;
        } finally {
            this._initPromise = null;
        }
    }
    
    /**
     * Load translations from locale files.
     */
    private async loadTranslations(): Promise<void> {
        try {
            // Load English translations
            const enResponse = await fetch('/locales/en.json');
            const enTranslations = await enResponse.json();
            i18n.addResourceBundle('en', 'translation', enTranslations, true, true);
            
            // Load Japanese translations
            const jaResponse = await fetch('/locales/ja.json');
            const jaTranslations = await jaResponse.json();
            i18n.addResourceBundle('ja', 'translation', jaTranslations, true, true);
            
            console.log('[i18n] Translations loaded');
        } catch (error) {
            console.error('[i18n] Failed to load translations:', error);
            // Fallback to built-in minimal translations
            this.addFallbackTranslations();
        }
    }
    
    /**
     * Add fallback translations.
     */
    private addFallbackTranslations(): void {
        i18n.addResourceBundle('en', 'translation', {
            app: { title: 'mml2vgm Browser IDE' },
            menu: { file: 'File', edit: 'Edit', view: 'View' },
            messages: { welcome: 'Welcome to mml2vgm Browser IDE' },
        }, true, true);
        
        i18n.addResourceBundle('ja', 'translation', {
            app: { title: 'mml2vgm ブラウザIDE' },
            menu: { file: 'ファイル', edit: '編集', view: '表示' },
            messages: { welcome: 'mml2vgm ブラウザIDEへようこそ' },
        }, true, true);
    }
    
    // ========================================================================
    // Language Management
    // ========================================================================
    
    /**
     * Set the language.
     */
    public setLanguage(language: SupportedLanguage): Promise<void> {
        return new Promise((resolve) => {
            const actualLanguage = language === 'auto' ? detectLanguage() : language;
            
            i18n.changeLanguage(actualLanguage, (error) => {
                if (error) {
                    console.error('[i18n] Failed to change language:', error);
                } else {
                    this._language = language;
                    localStorage.setItem('mml2vgm-language', actualLanguage);
                    this.notifyListeners();
                    console.log(`[i18n] Language changed to: ${actualLanguage}`);
                }
                resolve();
            });
        });
    }
    
    /**
     * Get the current language.
     */
    public getLanguage(): SupportedLanguage {
        return this._language;
    }
    
    /**
     * Get the current i18next language.
     */
    public getCurrentLanguage(): string {
        return i18n.language;
    }
    
    /**
     * Get supported languages.
     */
    public getSupportedLanguages(): SupportedLanguage[] {
        return ['en', 'ja', 'auto'];
    }
    
    /**
     * Get language display name.
     */
    public getLanguageName(lang: SupportedLanguage): string {
        const names: Record<SupportedLanguage, string> = {
            en: 'English',
            ja: '日本語',
            auto: 'Auto (Browser Default)',
        };
        return names[lang] || lang;
    }
    
    // ========================================================================
    // Translation Helpers
    // ========================================================================
    
    /**
     * Translate a key.
     */
    public t(key: string, options?: object): string {
        return i18n.t(key, options);
    }
    
    /**
     * Check if a translation key exists.
     */
    public has(key: string): boolean {
        return i18n.exists(key);
    }
    
    /**
     * Get the i18next instance.
     */
    public getI18n(): typeof i18n {
        return i18n;
    }
    
    // ========================================================================
    // State Management
    // ========================================================================
    
    /**
     * Get current state.
     */
    public getState(): I18nServiceState {
        return {
            language: this._language,
            isReady: this._isReady,
            error: this._error,
            supportedLanguages: this.getSupportedLanguages(),
        };
    }
    
    /**
     * Check if i18n is ready.
     */
    public get isReady(): boolean {
        return this._isReady;
    }
    
    /**
     * Get current error.
     */
    public get error(): string | null {
        return this._error;
    }
    
    /**
     * Subscribe to state changes.
     */
    public subscribe(callback: (state: I18nServiceState) => void): () => void {
        this.stateListeners.push(callback);
        callback(this.getState());
        
        return () => {
            const index = this.stateListeners.indexOf(callback);
            if (index >= 0) {
                this.stateListeners.splice(index, 1);
            }
        };
    }
    
    private notifyListeners(): void {
        this.stateListeners.forEach(callback => callback(this.getState()));
    }
}

// ============================================================================
// Singleton Instance
// ============================================================================

/**
 * Singleton instance of the I18nService.
 */
export const i18nService = I18nService.getInstance();

// ============================================================================
// Hook for React Components
// ============================================================================

import { useEffect, useState } from 'react';

/**
 * Hook to use translations in React components.
 */
export function useTranslation(): {
    t: (key: string, options?: object) => string;
    i18n: typeof i18n;
    language: SupportedLanguage;
    setLanguage: (lang: SupportedLanguage) => Promise<void>;
    isReady: boolean;
} {
    const service = I18nService.getInstance();
    const [state, setState] = useState(service.getState());
    
    useEffect(() => {
        const unsubscribe = service.subscribe(setState);
        return () => unsubscribe();
    }, []);
    
    return {
        t: service.t.bind(service),
        i18n: service.getI18n(),
        language: state.language,
        setLanguage: service.setLanguage.bind(service),
        isReady: state.isReady,
    };
}

// ============================================================================
// i18n Configuration for Direct Import
// ============================================================================

// This allows components to import and use i18n directly
declare module 'react-i18next' {
    interface CustomTypeOptions {
        defaultNS: 'translation';
        resources: {
            translation: typeof import('../public/locales/en.json');
        };
    }
}

export default I18nService;
