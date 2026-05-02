/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification of the Global System Settings and Persistent Environment store.** 
 * Tests the rehydration of configuration state from `localStorage`, validation of engine URLs and API keys, and the snapshotting of theme/density preferences. 
 * Mocks `localStorage` and `btoa/atob` to isolate persistence logic from environment-specific side-effects and ensuring consistent state recovery.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Configuration drift when local settings fail to synchronize with the backend `config.yaml` or failure to apply reactive UI updates on theme change.
 * - **Telemetry Link**: Search `[settings_store.test]` in tracing logs.
 */


/**
 * @file settings_store.test.ts
 * @description Suite for the Persistent System configuration store.
 * @module Stores/SettingsStore
 * @testedBehavior
 * - Rehydration: Manual trigger of persist.rehydrate() to wait for state recovery.
 * - Validation: Verification of URL and API key format constraints.
 * - Persistence: Snapshotting of engine URLs and theme preferences.
 * @aiContext
 * - Uses vi.hoisted to stub global.localStorage before any module re-evaluation.
 * - Manages an internal state for the localStorage mock to ensure rehydration consistency.
 * - Refactored for 100% snake_case architectural parity.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

vi.hoisted(() => {
    const mock_impl = {
        getItem: (key: string) => (global as any).__MOCK_STORAGE__?.[key] || null,
        setItem: (key: string, val: string) => { (global as any).__MOCK_STORAGE__ = { ...((global as any).__MOCK_STORAGE__ || {}), [key]: val }; },
        clear: () => { (global as any).__MOCK_STORAGE__ = {}; },
        removeItem: (key: string) => { delete (global as any).__MOCK_STORAGE__?.[key]; },
        length: 0,
        key: vi.fn(),
    };
    vi.stubGlobal('localStorage', mock_impl);
    // Stub atob/btoa for the environment if not present
    if (typeof btoa === 'undefined') {
        vi.stubGlobal('btoa', (str: string) => Buffer.from(str, 'binary').toString('base64'));
        vi.stubGlobal('atob', (str: string) => Buffer.from(str, 'base64').toString('binary'));
    }
});

describe('settings_store', () => {
    beforeEach(async () => {
        (global as any).__MOCK_STORAGE__ = {};
        vi.resetModules();
        vi.clearAllMocks();
    });

    afterEach(() => {
        delete (global as any).__MOCK_STORAGE__;
    });

    it('initializes with default value if no storage exists', async () => {
        const { get_settings, use_settings_store } = await import('./settings_store');
        await use_settings_store.persist.rehydrate();
        
        const settings = get_settings();
        expect(settings.tadpole_os_url).toBe('http://127.0.0.1:8000');
        expect(settings.tadpole_os_api_key).toBe(''); // Now expects empty string
    });

    it('rehydrates from localStorage correctly', async () => {
        const test_key = 'custom-key';

        const mock_saved_settings = {
            state: {
                settings: {
                    tadpole_os_url: 'http://custom-engine:9000',
                    tadpole_os_api_key: test_key,
                    theme: 'zinc',
                    density: 'comfortable',
                    default_model: 'GPT-4o',
                    default_temperature: 0.8,
                    auto_approve_safe_skills: false,
                    max_agents: 50,
                    max_clusters: 5,
                    max_swarm_depth: 3,
                    max_task_length: 1000,
                    default_budget_usd: 5,
                    is_safe_mode: false,
                    privacy_mode: true
                }
            },
            version: 0
        };

        (global as any).__MOCK_STORAGE__['tadpole_settings'] = JSON.stringify(mock_saved_settings);

        const { get_settings, use_settings_store } = await import('./settings_store');
        await use_settings_store.persist.rehydrate();

        const settings = get_settings();
        expect(settings.tadpole_os_url).toBe('http://custom-engine:9000');
        expect(settings.tadpole_os_api_key).toBe(test_key);
        expect(settings.privacy_mode).toBe(true);
    });


    it('validates settings before saving', async () => {
        const { save_settings, use_settings_store } = await import('./settings_store');
        await use_settings_store.persist.rehydrate();

        // Test invalid URL
        const err_url = save_settings({ tadpole_os_url: 'invalid-url' } as any);
        expect(err_url).toBe('Invalid URL. Must start with http:// or https://');

        // Test missing API key
        const err_key = save_settings({
            tadpole_os_url: 'http://valid',
            tadpole_os_api_key: '   ',
        } as any);
        expect(err_key).toBe('API token is required. Generate a NEURAL_TOKEN and paste it here.');

        // Test successful save
        const valid_settings = {
            tadpole_os_url: 'http://valid',
            tadpole_os_api_key: 'valid-key',
            theme: 'zinc',
            density: 'compact'
        };
        const res = save_settings(valid_settings as any);
        expect(res).toBeNull();
    });

    it('strips legacy placeholder tokens during rehydration', async () => {
        const mock_saved_settings = {
            state: {
                settings: {
                    tadpole_os_url: 'http://127.0.0.1:8000',
                    tadpole_os_api_key: 'my-secure-token-123',
                }
            },
            version: 0
        };

        (global as any).__MOCK_STORAGE__['tadpole_settings'] = JSON.stringify(mock_saved_settings);

        const { get_settings, use_settings_store } = await import('./settings_store');
        await use_settings_store.persist.rehydrate();

        // Legacy token is now stripped to empty string
        expect(get_settings().tadpole_os_api_key).toBe('');
    });

    it('allows default development tokens', async () => {
        const { is_valid_api_key } = await import('./settings_store');
        
        expect(is_valid_api_key('tadpole-dev-token-2026')).toBe(true);
        expect(is_valid_api_key('tadpole-os-sidecar-default-2026')).toBe(true);
    });
});


// Metadata: [settings_store_test]

// Metadata: [settings_store_test]
