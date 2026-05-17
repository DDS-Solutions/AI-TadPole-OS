/**
 * @docs ARCHITECTURE:Quality:Verification
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[setup]` in observability traces.
 */

import '@testing-library/jest-dom';
import { vi } from 'vitest';
import en_translations from '../src/locales/en/index';

// Mock HTMLCanvasElement.prototype.getContext
HTMLCanvasElement.prototype.getContext = (() => {
    return {} as unknown as RenderingContext;
}) as unknown as typeof HTMLCanvasElement.prototype.getContext;

// Mock localStorage
const local_storage_mock = (() => {
    let store: Record<string, string> = {};
    return {
        getItem: (key: string) => store[key] || null,
        setItem: (key: string, value: string) => {
            store[key] = value.toString();
        },
        removeItem: (key: string) => {
            delete store[key];
        },
        clear: () => {
            store = {};
        },
    };
})();

Object.defineProperty(window, 'localStorage', {
    value: local_storage_mock,
});

// Mock i18n
const namespaceMap: Record<string, string> = {
    'agent_card': 'agent',
    'agent_config': 'agent',
    'agent_manager': 'agent',
    'agent_role_select': 'agent',
    'agent_metrics': 'agent',
    'agent_details': 'agent',
    'engine_dashboard': 'system',
    'metrics': 'system',
    'stats': 'system',
    'telemetry': 'system',
    'telemetry_graph': 'system',
    'settings': 'system',
    'benchmark': 'system',
    'layout': 'system',
    'hardware': 'system',
    'missions': 'mission',
    'scheduled_jobs': 'mission',
    'workspaces': 'mission',
    'standups': 'mission',
    'transcript': 'mission',
    'voice': 'mission',
    'sidebar': 'nav',
    'docs': 'nav',
    'provider': 'intelligence',
    'skills': 'intelligence',
    'model_store': 'intelligence',
    'template_store': 'intelligence',
    'model_manager': 'intelligence',
    'chat': 'interface',
    'swarm_visualizer': 'interface',
    'oversight': 'security',
    'trace': 'observability',
    'trace_stream': 'observability',
    'terminal': 'observability',
    'system_log': 'observability'
};

vi.mock('../src/i18n', () => ({
    i18n: {
        t: (key: string, options?: any) => {
            let keys = key.split('.');
            
            // Apply Namespace Routing
            if (keys.length > 0 && namespaceMap[keys[0]]) {
                const newNamespace = namespaceMap[keys[0]];
                if (newNamespace !== keys[0]) {
                    keys = [newNamespace, ...keys];
                }
            }

            // Resolve Path
            let t: any = en_translations;
            for (const k of keys) {
                if (t && typeof t === 'object' && k in t) {
                    t = t[k];
                } else {
                    t = key;
                    break;
                }
            }

            if (typeof t !== 'string') t = key;

            // Interpolate
            if (options && typeof options === 'object') {
                Object.keys(options).forEach(opt_key => {
                    const val = String(options[opt_key]);
                    // Support both new {{param:key}} and legacy {{key}}
                    t = t.replace(new RegExp(`{{param:${opt_key}}}`, 'g'), val);
                    t = t.replace(new RegExp(`{{${opt_key}}}`, 'g'), val);
                });
            }
            return t;
        },
        language: 'en',
    },
}));

// Mock matchMedia for Framer Motion
Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: vi.fn().mockImplementation(query => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: vi.fn(), // deprecated
        removeListener: vi.fn(), // deprecated
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
    })),
});

// ResizeObserver mock for ReactFlow and layout-dependent tests
class Resize_Observer_Mock {
    observe() {}
    unobserve() {}
    disconnect() {}
}

window.ResizeObserver = Resize_Observer_Mock as any;
global.ResizeObserver = Resize_Observer_Mock as any;

// Mock BroadcastChannel for cross-tab synchronization
class Broadcast_Channel_Mock {
    name: string;
    onmessage: ((ev: MessageEvent) => void) | null = null;
    constructor(name: string) { this.name = name; }
    postMessage(data: any) {
        // Emit locally for tests that check incoming sync
        setTimeout(() => {
            if (this.onmessage) this.onmessage({ data } as MessageEvent);
        }, 0);
    }
    close() {}
}

(window as any).BroadcastChannel = Broadcast_Channel_Mock;
(global as any).BroadcastChannel = Broadcast_Channel_Mock;


// Metadata: [setup]
