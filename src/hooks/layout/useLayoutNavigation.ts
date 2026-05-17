import { useEffect } from 'react';
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

    // ── Tab/URL Synchronization ──────────────────────────
    useEffect(() => {
        // Guard: Detached windows should never trigger a navigation update
        if (location.pathname.startsWith('/detached')) {
            return;
        }

        const active_tab = (tabs || []).find(t => t.id === active_tab_id);
        if (active_tab && active_tab.path !== location.pathname) {
            navigate(active_tab.path);
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
