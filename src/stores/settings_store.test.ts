/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / settings_store.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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

    const mock_session_impl = {
        getItem: (key: string) => (global as any).__MOCK_SESSION_STORAGE__?.[key] || null,
        setItem: (key: string, val: string) => { (global as any).__MOCK_SESSION_STORAGE__ = { ...((global as any).__MOCK_SESSION_STORAGE__ || {}), [key]: val }; },
        clear: () => { (global as any).__MOCK_SESSION_STORAGE__ = {}; },
        removeItem: (key: string) => { delete (global as any).__MOCK_SESSION_STORAGE__?.[key]; },
        length: 0,
        key: vi.fn(),
    };
    vi.stubGlobal('sessionStorage', mock_session_impl);

    // Stub atob/btoa for the environment if not present
    if (typeof btoa === 'undefined') {
        vi.stubGlobal('btoa', (str: string) => Buffer.from(str, 'binary').toString('base64'));
        vi.stubGlobal('atob', (str: string) => Buffer.from(str, 'base64').toString('binary'));
    }
});

describe('settings_store', () => {
    beforeEach(async () => {
        (global as any).__MOCK_STORAGE__ = {};
        (global as any).__MOCK_SESSION_STORAGE__ = {};
        vi.resetModules();
        vi.clearAllMocks();
        vi.stubEnv('VITE_NEURAL_TOKEN', '');
    });

    afterEach(() => {
        delete (global as any).__MOCK_STORAGE__;
        delete (global as any).__MOCK_SESSION_STORAGE__;
        vi.unstubAllEnvs();
    });

    it('initializes with default value if no storage exists', async () => {
        const { get_settings, use_settings_store } = await import('./settings_store');
        await use_settings_store.persist.rehydrate();
        
        const settings = get_settings();
        expect(settings.tadpole_os_url).toBe('http://127.0.0.1:8000');
        expect(settings.tadpole_os_api_key).toBe(''); // Now expects empty string
    });

    it('rehydrates from localStorage correctly with session token recovery', async () => {
        const test_key = 'custom-valid-neural-token-1234';

        const mock_saved_settings = {
            state: {
                settings: {
                    tadpole_os_url: 'http://custom-engine:9000',
                    tadpole_os_api_key: '',
                    allowed_origins: ['http://10.0.0.1:8000'],
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
        (global as any).__MOCK_SESSION_STORAGE__['tadpole_session_token'] = test_key;

        const { get_settings, use_settings_store } = await import('./settings_store');
        await use_settings_store.persist.rehydrate();

        const settings = get_settings();
        expect(settings.tadpole_os_url).toBe('http://custom-engine:9000');
        expect(settings.tadpole_os_api_key).toBe(test_key);
        expect(settings.allowed_origins).toEqual(['http://10.0.0.1:8000']);
        expect(settings.privacy_mode).toBe(true);
    });


    it('validates settings before saving', async () => {
        const { save_settings, use_settings_store } = await import('./settings_store');
        await use_settings_store.persist.rehydrate();

        // Test invalid URL
        const err_url = save_settings({ tadpole_os_url: 'invalid-url' } as any);
        expect(err_url).toBe('Invalid URL. Must start with http:// or https://');

        // Test missing or too-short API key (< 16 chars)
        const err_key_short = save_settings({
            tadpole_os_url: 'http://valid',
            tadpole_os_api_key: 'short-key',
        } as any);
        expect(err_key_short).toBe('API token is required and must be at least 16 characters. Generate a NEURAL_TOKEN and paste it here.');

        // Test missing API key
        const err_key = save_settings({
            tadpole_os_url: 'http://valid',
            tadpole_os_api_key: '   ',
        } as any);
        expect(err_key).toBe('API token is required and must be at least 16 characters. Generate a NEURAL_TOKEN and paste it here.');

        // Test successful save (>= 16 chars)
        const valid_settings = {
            tadpole_os_url: 'http://valid',
            tadpole_os_api_key: 'valid-neural-token-1234',
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

    it('validates minimum token length for development tokens', async () => {
        const { is_valid_api_key } = await import('./settings_store');
        
        expect(is_valid_api_key('tadpole-dev-token-2026')).toBe(true);
        expect(is_valid_api_key('tadpole-os-sidecar-default-2026')).toBe(true);
        expect(is_valid_api_key('too-short')).toBe(false);
    });

    it('partialize strips tadpole_os_api_key from localStorage persistence at rest', async () => {
        const { use_settings_store } = await import('./settings_store');
        
        use_settings_store.setState({
            settings: {
                tadpole_os_url: 'http://127.0.0.1:8000',
                tadpole_os_api_key: 'super-secret-neural-token-1234',
                allowed_origins: ['http://10.0.0.1:8000'],
                theme: 'dark',
                density: 'comfortable',
                default_model: 'gpt-4o',
                default_provider: 'openai',
                default_temperature: 1.0,
                auto_approve_safe_skills: false,
                max_agents: 10,
                max_clusters: 5,
                max_swarm_depth: 5,
                max_task_length: 10,
                default_budget_usd: 10.0,
                is_safe_mode: false,
                privacy_mode: false
            }
        });

        const persistOptions = (use_settings_store as any).persist?.getOptions?.();
        if (persistOptions?.partialize) {
            const persisted = persistOptions.partialize(use_settings_store.getState());
            expect(persisted.settings.tadpole_os_api_key).toBe('');
            expect(persisted.settings.tadpole_os_url).toBe('http://127.0.0.1:8000');
            expect(persisted.settings.allowed_origins).toEqual(['http://10.0.0.1:8000']);
        }
    });
});
