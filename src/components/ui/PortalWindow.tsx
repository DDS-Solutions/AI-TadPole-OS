import React, { useEffect, useState, useMemo } from 'react';
import { createPortal } from 'react-dom';

interface PortalWindowProps {
    children: React.ReactNode;
    title: string;
    onClose: () => void;
    id: string;
}

/**
 * PortalWindow
 * Renders its children into a detached browser window while maintaining
 * the React component tree and state of the parent instance.
 */
export const PortalWindow: React.FC<PortalWindowProps> = ({ children, title, onClose, id }) => {
    const [container, setContainer] = useState<HTMLDivElement | null>(null);
    const [externalWindow, setExternalWindow] = useState<Window | null>(null);

    // Use a ref for onClose to avoid re-running the setup effect if the callback changes
    const onCloseRef = React.useRef(onClose);
    onCloseRef.current = onClose;

    const titleRef = React.useRef(title);
    titleRef.current = title;

    // Unique window name to prevent collisions but allow window re-use for the same tab
    const windowName = useMemo(() => `tadpole-detached-${id}`, [id]);

    useEffect(() => {
        // 1. Open new window
        const features = 'width=1000,height=700,left=100,top=100,resizable=yes,scrollbars=yes,status=no,menubar=no,toolbar=no';
        const win = window.open('', windowName, features);

        if (!win) {
            console.error('[PortalWindow] Popup blocked or failed to open.');
            onCloseRef.current();
            return;
        }

        setExternalWindow(win);

        // 2. Prepare child document
        const childDoc = win.document;
        childDoc.title = `${titleRef.current} | Tadpole OS Detached`;
        
        // Clear anything that might be there if we are re-using the window
        childDoc.body.innerHTML = '';
        
        // Apply basic background to prevent white flash before styles load
        childDoc.body.style.backgroundColor = '#09090b'; // zinc-950
        childDoc.body.style.margin = '0';
        childDoc.body.style.overflow = 'hidden';

        const rootContainer = childDoc.createElement('div');
        rootContainer.id = 'detached-root';
        rootContainer.style.height = '100vh';
        childDoc.body.appendChild(rootContainer);
        setContainer(rootContainer);

        // 3. Sync Styles logic
        const syncStyles = () => {
            // Remove old styles
            Array.from(childDoc.head.querySelectorAll('link[rel="stylesheet"], style')).forEach(el => el.remove());
            
            // Inject current styles
            Array.from(document.styleSheets).forEach((styleSheet) => {
                try {
                    if (styleSheet.href) {
                        const newLink = childDoc.createElement('link');
                        newLink.rel = 'stylesheet';
                        newLink.href = styleSheet.href;
                        childDoc.head.appendChild(newLink);
                    } else if (styleSheet.cssRules) {
                        const newStyle = childDoc.createElement('style');
                        Array.from(styleSheet.cssRules).forEach((rule) => {
                            newStyle.appendChild(childDoc.createTextNode(rule.cssText));
                        });
                        childDoc.head.appendChild(newStyle);
                    }
                } catch (e) {
                    // Ignore CORS issues with external stylesheets
                    console.debug('[PortalWindow] Could not sync some styles:', e);
                }
            });
        };

        syncStyles();

        // 4. Style Mutation Observer for HMR and lazy Tailwind styles
        const observer = new MutationObserver(() => syncStyles());
        observer.observe(document.head, { childList: true, subtree: true });

        // 5. Handle manual window closure
        const handleUnload = () => {
            onCloseRef.current();
        };
        win.addEventListener('unload', handleUnload);

        // 6. Focus the new window
        win.focus();

        return () => {
            observer.disconnect();
            win.removeEventListener('unload', handleUnload);
            win.close();
            setExternalWindow(null);
            setContainer(null);
        };
    }, [windowName]); // Only re-run if windowName (derived from id) changes

    // Update title if it changes in store
    useEffect(() => {
        if (externalWindow) {
            // eslint-disable-next-line react-hooks/immutability
            externalWindow.document.title = `${title} | Tadpole OS Detached`;
        }
    }, [title, externalWindow]);

    if (!container) return null;

    return createPortal(
        <div className="h-full bg-zinc-950 text-zinc-100 selection:bg-zinc-700/30 font-sans antialiased overflow-hidden">
            {children}
        </div>,
        container
    );
};
