import React, { useEffect, useState, useMemo, useRef } from 'react';
import { createPortal } from 'react-dom';

/**
 * Portal_Window_Props
 * Defines the strict interface for the portal window component.
 */
interface Portal_Window_Props {
    children: React.ReactNode;
    title: string;
    on_close: () => void;
    id: string;
}

/**
 * Portal_Window
 * Renders its children into a detached browser window while maintaining
 * the React component tree and state of the parent instance.
 */
export const Portal_Window: React.FC<Portal_Window_Props> = ({ children, title, on_close, id }) => {
    const [container, set_container] = useState<HTMLDivElement | null>(null);
    const [external_window, set_external_window] = useState<Window | null>(null);

    // Use a ref for on_close to avoid re-running the setup effect if the callback changes
    const on_close_ref = useRef(on_close);
    on_close_ref.current = on_close;

    const title_ref = useRef(title);
    title_ref.current = title;

    // Unique window name to prevent collisions but allow window re-use for the same tab
    const window_name = useMemo(() => `tadpole-detached-${id}`, [id]);

    useEffect(() => {
        // 1. Open new window
        const features = 'width=1000,height=700,left=100,top=100,resizable=yes,scrollbars=yes,status=no,menubar=no,toolbar=no';
        const win = window.open('', window_name, features);

        if (!win) {
            console.error('[Portal_Window] Popup blocked or failed to open.');
            on_close_ref.current();
            return;
        }

        set_external_window(win);

        // 2. Prepare child document
        const child_doc = win.document;
        child_doc.title = `${title_ref.current} | Tadpole OS Detached`;
        
        // Clear anything that might be there if we are re-using the window
        child_doc.body.innerHTML = '';
        
        // Apply basic background to prevent white flash before styles load
        child_doc.body.style.backgroundColor = '#09090b'; // zinc-950
        child_doc.body.style.margin = '0';
        child_doc.body.style.overflow = 'hidden';

        const root_container = child_doc.createElement('div');
        root_container.id = 'detached-root';
        root_container.style.height = '100vh';
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
    }, [window_name]); // Only re-run if window_name (derived from id) changes

    // Update title if it changes in store
    useEffect(() => {
        if (external_window) {
            // eslint-disable-next-line react-hooks/immutability
            external_window.document.title = `${title} | Tadpole OS Detached`;
        }
    }, [title, external_window]);

    if (!container) return null;

    return createPortal(
        <div className="h-full bg-zinc-950 text-zinc-100 selection:bg-zinc-700/30 font-sans antialiased overflow-hidden">
            {children}
        </div>,
        container
    );
};
