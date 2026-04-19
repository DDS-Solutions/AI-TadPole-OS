/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Multi-window orchestration manager (Sector Detachment). 
 * Detaches React component sub-trees into independent browser windows while maintaining shared context, style parity (via MutationObserver), and lifecycle sync.
 * 
 * ### 🧬 Logic Flow (Mermaid)
 * ```mermaid
 * sequenceDiagram
 *     participant P as Parent Tab
 *     participant W as Child Window
 *     participant MO as MutationObserver
 * 
 *     P->>W: window.open()
 *     P->>W: Inject Styles (sync_styles)
 *     P->>MO: observe(document.head)
 *     MO-->>W: Update Styles on HMR
 *     W->>P: onUnload -> on_close()
 *     P->>W: unmount -> window.close()
 * ```
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Popup blocked by browser policy, style sync failure on HMR update, or zombie window if `on_close` isn't triggered during tab closure.
 * - **Telemetry Link**: Search for `[Portal_Window]` or `tadpole-detached` in browser logs.
 */

import React, { useEffect, useState, useMemo, useRef } from 'react';
import { createPortal } from 'react-dom';
import { i18n } from '../../i18n';

/**
 * Portal_Window_Props
 * Defines the strict interface for the portal window component.
 */
interface Portal_Window_Props {
    children: React.ReactNode;
    title: string;
    on_close: () => void;
    on_popup_block?: () => void;
    id: string;
    width?: number;
    height?: number;
    url?: string;
}

/**
 * Portal_Window
 * Renders its children into a detached browser window while maintaining
 * the React component tree and state of the parent instance.
 */
export const Portal_Window: React.FC<Portal_Window_Props> = ({ 
    children, 
    title, 
    on_close,
    on_popup_block, 
    id,
    width = 440,
    height = 720,
    url
}) => {
    const [container, set_container] = useState<HTMLDivElement | null>(null);
    const [external_window, set_external_window] = useState<Window | null>(null);

    // Use a ref for on_close to avoid re-running the setup effect if the callback changes
    const on_close_ref = useRef(on_close);
    on_close_ref.current = on_close;

    const on_popup_block_ref = useRef(on_popup_block);
    on_popup_block_ref.current = on_popup_block;

    const title_ref = useRef(title);
    title_ref.current = title;

    // Unique window name to prevent collisions but allow window re-use for the same tab
    const window_name = useMemo(() => `tadpole-detached-${id}`, [id]);

    useEffect(() => {
        // --- TAURI NATIVE DETACHMENT BRIDGE ---
        // In Tauri production, createPortal across windows fails because they are isolated webview processes.
        // We bypass this by opening a native window pointing to the specific component route.
        const is_tauri = !!(window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__;
        
        if (is_tauri && url) {
            const open_tauri_window = async () => {
                const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
                const win = new WebviewWindow(window_name, {
                    url: url,
                    title: `${title_ref.current} | ${i18n.t('common.detached_suffix')}`,
                    width,
                    height,
                    resizable: true,
                    decorations: true,
                    transparent: false
                });

                win.once('tauri://error', (e) => {
                    console.error('[Portal_Window] Tauri window error:', e);
                    on_close_ref.current();
                });

                win.once('tauri://close-requested', () => {
                    on_close_ref.current();
                });
            };

            open_tauri_window();
            return () => {
                // We don't necessarily want to force-close the native window on unmount 
                // if the user expects it to stay open, but for Portal parity we should.
                const close_tauri_window = async () => {
                   const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
                   const win = await WebviewWindow.getByLabel(window_name);
                   await win?.close();
                };
                close_tauri_window();
            };
        }

        // --- BROWSER PORTAL FALLBACK (DEV / BROWSER) ---
        const features = `width=${width},height=${height},left=100,top=100,resizable=yes,scrollbars=yes,status=no,menubar=no,toolbar=no,location=no,directories=no`;
        const win = window.open('', window_name, features);

        if (!win) {
            console.error('[Portal_Window] Popup blocked or failed to open.');
            on_popup_block_ref.current?.();
            on_close_ref.current();
            return;
        }

        set_external_window(win);

        // 2. Prepare child document
        const child_doc = win.document;
        child_doc.title = `${title_ref.current} | ${i18n.t('common.detached_suffix')}`;
        
        // Clear anything that might be there if we are re-using the window
        child_doc.body.replaceChildren();
        
        // 2. Clear default styles and set normalization
        child_doc.documentElement.style.height = '100%';
        child_doc.documentElement.style.width = '100%';
        child_doc.body.style.margin = '0';
        child_doc.body.style.padding = '0';
        child_doc.body.style.overflow = 'hidden';
        child_doc.body.style.height = '100%';
        child_doc.body.style.width = '100%';
        child_doc.body.style.backgroundColor = '#09090b'; // zinc-950

        // 3. Create root container
        const root_container = child_doc.createElement('div');
        root_container.id = 'detached-root';
        root_container.style.height = '100%';
        root_container.style.width = '100%';
        root_container.style.display = 'flex';
        root_container.style.flexDirection = 'column';
        root_container.style.overflow = 'hidden';
        child_doc.body.appendChild(root_container);
        set_container(root_container);

        // 3. Sync Styles logic
        const sync_styles = () => {
            // Remove old styles
            Array.from(child_doc.head.querySelectorAll('link[rel="stylesheet"], style')).forEach(el => el.remove());
            
            // Inject current styles
            Array.from(document.styleSheets).forEach((style_sheet) => {
                try {
                    if (style_sheet.href) {
                        const new_link = child_doc.createElement('link');
                        new_link.rel = 'stylesheet';
                        new_link.href = style_sheet.href;
                        child_doc.head.appendChild(new_link);
                    } else if (style_sheet.cssRules) {
                        const new_style = child_doc.createElement('style');
                        Array.from(style_sheet.cssRules).forEach((rule) => {
                            new_style.appendChild(child_doc.createTextNode(rule.cssText));
                        });
                        child_doc.head.appendChild(new_style);
                    }
                } catch (e) {
                    // Ignore CORS issues with external stylesheets
                    console.debug('[Portal_Window] Could not sync some styles:', e);
                }
            });
        };

        sync_styles();

        // 4. Style Mutation Observer for HMR and lazy Tailwind styles
        const observer = new MutationObserver(() => sync_styles());
        observer.observe(document.head, { childList: true, subtree: true });

        // 5. Handle manual window closure
        const handle_unload = () => {
            on_close_ref.current();
        };
        win.addEventListener('unload', handle_unload);

        // 6. Focus the new window
        win.focus();

        return () => {
            observer.disconnect();
            win.removeEventListener('unload', handle_unload);
            win.close();
            set_external_window(null);
            set_container(null);
        };
    }, [window_name, width, height, url]); // Only re-run if window name (id), dimensions, or URL change

    useEffect(() => {
        if (external_window) {
            external_window.document.title = `${title} | ${i18n.t('common.detached_suffix')}`;
        }
    }, [title, external_window]);

    if (!container) return null;

    return createPortal(
        <div className="w-full h-full bg-zinc-950 text-zinc-100 selection:bg-zinc-700/30 font-sans antialiased overflow-hidden flex flex-col">
            {children}
        </div>,
        container
    );
};

