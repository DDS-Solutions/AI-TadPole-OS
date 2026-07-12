/**
 * @docs ARCHITECTURE:UI-Hooks
 * 
 * ### AI Assist Note
 * **useLayoutNavigation**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[useLayoutNavigation]` in observability traces.
 */

import { useEffect, useRef } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { use_tab_store } from '../../stores/tab_store';

/**
 * useLayoutNavigation
 * Manages tab-to-route synchronization and global keyboard navigation shortcuts.
 */
export function useLayoutNavigation(set_is_command_palette_open: (open: boolean | ((prev: boolean) => boolean)) => void) {
    const location = useLocation();
    const navigate = useNavigate();
    const { tabs, active_tab_id } = use_tab_store();

    // Keep track of the last active tab ID to distinguish between tab clicks and NavLink clicks
    const prev_active_tab_id = useRef(active_tab_id);

    // ── Tab/URL Synchronization ──────────────────────────
    useEffect(() => {
        // Guard: Detached windows should never trigger a navigation update
        if (location.pathname.startsWith('/detached')) {
            return;
        }

        const active_tab = (tabs || []).find(t => t.id === active_tab_id);
        
        // Only force a navigation if the active tab changed (e.g. user clicked a Tab in the Tab_Bar)
        if (active_tab && prev_active_tab_id.current !== active_tab_id) {
            prev_active_tab_id.current = active_tab_id;
            if (active_tab.path !== location.pathname) {
                navigate(active_tab.path);
            }
        } else if (active_tab && active_tab.path === location.pathname) {
            // Keep ref in sync
            prev_active_tab_id.current = active_tab_id;
        }
    }, [active_tab_id, location.pathname, navigate, tabs]);

    // ── Keyboard Shortcuts ───────────────────────────────
    useEffect(() => {
        const handle_key_down = (e: KeyboardEvent) => {
            const tag = (e.target as HTMLElement)?.tagName;
            if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;

            // Command Palette (Ctrl+K or Ctrl+/)
            if ((e.ctrlKey || e.metaKey) && (e.key === 'k' || e.key === '/')) {
                e.preventDefault();
                set_is_command_palette_open(prev => !prev);
                return;
            }

            // Quick Navigation (1-6)
            if (!e.ctrlKey && !e.metaKey && !e.altKey) {
                const routes: Record<string, string> = {
                    '1': '/',
                    '2': '/org-chart',
                    '3': '/standups',
                    '4': '/workspaces',
                    '5': '/docs',
                    '6': '/settings',
                };
                if (routes[e.key]) {
                    navigate(routes[e.key]);
                    return;
                }
            }
        };

        window.addEventListener('keydown', handle_key_down);
        return () => window.removeEventListener('keydown', handle_key_down);
    }, [navigate, set_is_command_palette_open]);
}

// Metadata: [useLayoutNavigation]
