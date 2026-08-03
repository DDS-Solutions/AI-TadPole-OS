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

import React, { useEffect, useState, useMemo, useRef, useLayoutEffect } from 'react';
import { createPortal } from 'react-dom';
import { Minimize2 } from 'lucide-react';
import { i18n } from '../../i18n';

/**
 * Portal_Window_Props
 * Defines the flexible interface supporting both camelCase and snake_case props.
 */
interface Portal_Window_Props {
    children: React.ReactNode;
    title: string;
    on_close?: () => void;
    onClose?: () => void;
    on_popup_block?: () => void;
    onPopupBlock?: () => void;
    id: string;
    width?: number;
    height?: number;
    url?: string;
    auto_scale?: boolean;
    autoScale?: boolean;
}

/**
 * PortalWindow / Portal_Window
 * Renders its children into a detached browser window while maintaining
 * the React component tree and state of the parent instance.
 */
export const PortalWindow: React.FC<Portal_Window_Props> = ({ 
    children, 
    title, 
    on_close,
    onClose,
    on_popup_block,
    onPopupBlock,
    id,
    width = 1200,
    height = 800,
    url,
    auto_scale,
    autoScale
}) => {
    const is_auto_scale_enabled = autoScale ?? auto_scale ?? true;
    const [container, set_container] = useState<HTMLDivElement | null>(null);

    const close_fn = onClose || on_close || (() => {});
    const popup_block_fn = onPopupBlock || on_popup_block;

    // Use refs for callbacks to prevent setup effect re-runs when callbacks mutate
    const on_close_ref = useRef(close_fn);
    const on_popup_block_ref = useRef(popup_block_fn);
    const title_ref = useRef(title);
    const external_window_ref = useRef<Window | null>(null);

    useLayoutEffect(() => {
        on_close_ref.current = close_fn;
        on_popup_block_ref.current = popup_block_fn;
        title_ref.current = title;
    });

    const window_name = useMemo(() => `tadpole-detached-${id}`, [id]);

    useEffect(() => {
        // --- TAURI NATIVE DETACHMENT BRIDGE ---
        const is_tauri = typeof window !== 'undefined' && 
            (!!(window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ || 
             !!(window as unknown as { __TAURI__: unknown }).__TAURI__);
        
        if (is_tauri && url) {
            const open_tauri_window = async () => {
                const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
                const existing = await WebviewWindow.getByLabel(window_name);
                if (existing) {
                    await (existing as unknown as { setFocus?: () => Promise<void> }).setFocus?.();
                    return;
                }
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
                const close_tauri_window = async () => {
                   const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
                   const win = await WebviewWindow.getByLabel(window_name);
                   await win?.close();
                };
                close_tauri_window();
            };
        }

        // --- BROWSER PORTAL FALLBACK ---
        const features = `width=${width},height=${height},left=100,top=100,resizable=yes,scrollbars=yes,status=no,menubar=no,toolbar=no,location=no,directories=no`;
        const win = window.open('', window_name, features);

        if (!win) {
            console.error('[Portal_Window] Popup blocked or failed to open.');
            on_popup_block_ref.current?.();
            on_close_ref.current();
            return;
        }

        external_window_ref.current = win;
        const child_doc = win.document;

        // Debounced FOUC-Free Style Synchronization logic using rAF
        let sync_scheduled = false;
        const sync_styles = () => {
            if (sync_scheduled) return;
            sync_scheduled = true;
            requestAnimationFrame(() => {
                sync_scheduled = false;
                if (!win || win.closed) return;
                const existing_links = new Set(
                    Array.from(child_doc.head.querySelectorAll('link[rel="stylesheet"]'))
                        .map(el => (el as HTMLLinkElement).href)
                );

                Array.from(document.styleSheets).forEach((style_sheet) => {
                    try {
                        if (style_sheet.href) {
                            if (!existing_links.has(style_sheet.href)) {
                                const new_link = child_doc.createElement('link');
                                new_link.rel = 'stylesheet';
                                new_link.href = style_sheet.href;
                                child_doc.head.appendChild(new_link);
                            }
                            return;
                        } 
                        
                        if (style_sheet.cssRules) {
                            const css_text = Array.from(style_sheet.cssRules).map(r => r.cssText).join('\n');
                            const new_style = child_doc.createElement('style');
                            new_style.textContent = css_text;
                            child_doc.head.appendChild(new_style);
                        }
                    } catch {
                        // Ignore cross-origin stylesheet errors silently
                    }
                });
            });
        };

        const handle_close = () => {
            on_close_ref.current();
        };

        // Modern pagehide + beforeunload fallback event handling
        const attach_lifecycle_listeners = (target_win: Window) => {
            target_win.addEventListener('pagehide', handle_close);
            target_win.addEventListener('beforeunload', handle_close);
            target_win.addEventListener('unload', handle_close);
        };

        const detach_lifecycle_listeners = (target_win: Window) => {
            target_win.removeEventListener('pagehide', handle_close);
            target_win.removeEventListener('beforeunload', handle_close);
            target_win.removeEventListener('unload', handle_close);
        };

        // Heartbeat monitor for popup windows that get closed without firing unload/pagehide
        const heartbeat_interval = setInterval(() => {
            if (external_window_ref.current?.closed) {
                clearInterval(heartbeat_interval);
                handle_close();
            }
        }, 500);

        const existing_root = child_doc.getElementById('detached-root') as HTMLDivElement | null;
        if (existing_root) {
            win.focus();
            queueMicrotask(() => set_container(existing_root));
            
            attach_lifecycle_listeners(win);
            
            const observer = new MutationObserver(() => sync_styles());
            observer.observe(document.head, { childList: true, subtree: true });

            return () => {
                clearInterval(heartbeat_interval);
                observer.disconnect();
                detach_lifecycle_listeners(win);
                win.close();
                external_window_ref.current = null;
                set_container(null);
            };
        }

        child_doc.title = `${title_ref.current} | ${i18n.t('common.detached_suffix')}`;
        
        const sync_root_attributes = () => {
            const parent_html = document.documentElement;
            const theme = parent_html.getAttribute('data-theme') || 'zinc';
            const density = parent_html.getAttribute('data-density') || 'compact';
            child_doc.documentElement.setAttribute('data-theme', theme);
            child_doc.documentElement.setAttribute('data-density', density);
        };
        sync_root_attributes();
        
        child_doc.body.replaceChildren();
        
        child_doc.documentElement.style.height = '100%';
        child_doc.documentElement.style.width = '100%';
        child_doc.body.style.margin = '0';
        child_doc.body.style.padding = '0';
        child_doc.body.style.overflow = 'hidden';
        child_doc.body.style.height = '100%';
        child_doc.body.style.width = '100%';
        
        // Dynamic initial background color matching active theme to prevent FOUC
        const parent_bg = window.getComputedStyle(document.body).backgroundColor || '#09090b';
        child_doc.body.style.backgroundColor = parent_bg;

        const root_container = child_doc.createElement('div');
        root_container.id = 'detached-root';
        root_container.style.height = '100%';
        root_container.style.width = '100%';
        root_container.style.display = 'flex';
        root_container.style.flexDirection = 'column';
        root_container.style.overflow = 'hidden';
        root_container.style.backfaceVisibility = 'hidden';
        child_doc.body.appendChild(root_container);
        
        queueMicrotask(() => set_container(root_container));

        sync_styles();

        const observer = new MutationObserver(() => {
            sync_root_attributes();
            sync_styles();
        });
        observer.observe(document.head, { childList: true, subtree: true });
        observer.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme', 'data-density'] });

        // Dynamic CSS Auto-Scaling calculation based on detached window dimensions
        const update_auto_scale = () => {
            if (!win) return;
            const base_w = width || 1200;
            const base_h = height || 800;
            const current_w = win.innerWidth || win.document.documentElement.clientWidth || base_w;
            const current_h = win.innerHeight || win.document.documentElement.clientHeight || base_h;

            const scale_w = current_w / base_w;
            const scale_h = current_h / base_h;
            // Proportional scale factor bounded between 0.55 and 1.0 to prevent layout distortion while fitting small windows
            const scale = Math.min(1.0, Math.max(0.55, Math.min(scale_w, scale_h)));

            const root_style = root_container.style;
            if (typeof (root_style as unknown as { zoom?: string }).zoom !== 'undefined') {
                (root_style as unknown as { zoom: string }).zoom = scale.toFixed(3);
            } else {
                root_style.transform = `scale(${scale.toFixed(3)})`;
                root_style.transformOrigin = 'top left';
                root_style.width = `${(100 / scale).toFixed(2)}%`;
                root_style.height = `${(100 / scale).toFixed(2)}%`;
            }
        };

        if (is_auto_scale_enabled) {
            update_auto_scale();
            win.addEventListener('resize', update_auto_scale);
        }

        attach_lifecycle_listeners(win);
        win.focus();

        return () => {
            clearInterval(heartbeat_interval);
            observer.disconnect();
            if (is_auto_scale_enabled) {
                win.removeEventListener('resize', update_auto_scale);
            }
            detach_lifecycle_listeners(win);
            win.close();
            external_window_ref.current = null;
            set_container(null);
        };
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [window_name, url, is_auto_scale_enabled]); // Width and height handled in separate resize effect to prevent window restarts

    // Separate dedicated effect for smooth window resizing without unmounting/restarting
    useEffect(() => {
        if (external_window_ref.current && width && height) {
            try {
                external_window_ref.current.resizeTo(width, height);
            } catch {
                // Ignore browser security restrictions on cross-origin window resize
            }
        }
    }, [width, height]);

    // Synchronize title if changed
    useEffect(() => {
        if (external_window_ref.current) {
            external_window_ref.current.document.title = `${title} | ${i18n.t('common.detached_suffix')}`;
        }
    }, [title]);

    if (!container) return null;

    const has_close_callback = !!(onClose || on_close);

    return createPortal(
        <div className="w-full h-full bg-[color:var(--color-background)] text-zinc-100 selection:bg-zinc-700/30 font-sans antialiased overflow-hidden flex flex-col">
            {/* Cybernetic Detached Header Bar */}
            <header className="h-9 px-3 bg-zinc-950/95 border-b border-zinc-800/80 backdrop-blur-md flex items-center justify-between shrink-0 select-none z-50">
                <div className="flex items-center gap-2.5 min-w-0">
                    <span className="relative flex h-2 w-2 shrink-0">
                        <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
                        <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500" />
                    </span>
                    <span className="text-xs font-semibold text-zinc-200 truncate font-mono tracking-tight">
                        {title}
                    </span>
                    <span className="hidden sm:inline-block px-1.5 py-0.5 text-[8px] font-mono font-bold uppercase tracking-wider text-emerald-400 bg-emerald-500/10 border border-emerald-500/20 rounded">
                        PORTAL ACTIVE
                    </span>
                </div>

                <div className="flex items-center gap-2">
                    {has_close_callback && (
                        <button
                            onClick={() => on_close_ref.current()}
                            className="flex items-center gap-1.5 px-2 py-0.5 text-[10px] font-mono font-medium text-zinc-400 hover:text-zinc-100 bg-zinc-900/80 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 rounded transition-all active:scale-95 group"
                            title={i18n.t('chat.restore_tooltip') || 'Recall / Dock Window'}
                        >
                            <Minimize2 size={11} className="group-hover:-rotate-45 transition-transform" />
                            <span>RECALL</span>
                        </button>
                    )}
                </div>
            </header>

            {/* Main Detached Content */}
            <div className="flex-1 relative overflow-hidden flex flex-col">
                {children}
            </div>
        </div>,
        container
    );
};

export const Portal_Window = PortalWindow;
// Metadata: [Portal_Window]
