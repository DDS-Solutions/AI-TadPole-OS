/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Ui / Portal_Window
 * - **Primary Entrypoints**: `PortalWindow`, `Portal_Window`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[Portal_Window]`
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
function sync_document_styles(child_doc: Document) {
  const existing_links = new Set(
    Array.from(child_doc.head.querySelectorAll('link[rel="stylesheet"]'))
      .map(el => (el as HTMLLinkElement).href)
  );
  const existing_inline_styles = new Set(
    Array.from(child_doc.head.querySelectorAll('style'))
      .map(el => el.textContent || '')
  );

  if (document.adoptedStyleSheets && child_doc.adoptedStyleSheets) {
    try {
      child_doc.adoptedStyleSheets = [...document.adoptedStyleSheets];
    } catch {
      try {
        const new_sheets = Array.from(document.adoptedStyleSheets).map(sheet => {
          const new_sheet = new CSSStyleSheet();
          const rules = Array.from(sheet.cssRules).map(r => r.cssText).join('\n');
          new_sheet.replaceSync(rules);
          return new_sheet;
        });
        child_doc.adoptedStyleSheets = new_sheets;
      } catch {
        // ignore
      }
    }
  }

  Array.from(document.styleSheets).forEach((style_sheet) => {
    try {
      if (style_sheet.href) {
        if (!existing_links.has(style_sheet.href)) {
          const new_link = child_doc.createElement('link');
          new_link.rel = 'stylesheet';
          new_link.href = style_sheet.href;
          child_doc.head.appendChild(new_link);
          existing_links.add(style_sheet.href);
        }
        return;
      } 
      
      if (style_sheet.cssRules) {
        const css_text = Array.from(style_sheet.cssRules).map(r => r.cssText).join('\n');
        if (css_text && !existing_inline_styles.has(css_text)) {
          const new_style = child_doc.createElement('style');
          new_style.textContent = css_text;
          child_doc.head.appendChild(new_style);
          existing_inline_styles.add(css_text);
        }
      }
    } catch {
      // Ignore cross-origin stylesheet errors silently
    }
  });
}

function apply_window_scale(win: Window, root_container: HTMLElement, width: number, height: number) {
  const base_w = width || 1200;
  const base_h = height || 800;
  const current_w = win.innerWidth || win.document.documentElement.clientWidth || base_w;
  const current_h = win.innerHeight || win.document.documentElement.clientHeight || base_h;

  const scale_w = current_w / base_w;
  const scale_h = current_h / base_h;
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
}

function init_child_document(child_doc: Document, title: string): HTMLDivElement {
  child_doc.title = `${title} | ${i18n.t('common.detached_suffix')}`;
  
  const parent_html = document.documentElement;
  child_doc.documentElement.setAttribute('data-theme', parent_html.getAttribute('data-theme') || 'zinc');
  child_doc.documentElement.setAttribute('data-density', parent_html.getAttribute('data-density') || 'compact');
  
  child_doc.body.replaceChildren();
  child_doc.documentElement.style.height = '100%';
  child_doc.documentElement.style.width = '100%';
  child_doc.body.style.margin = '0';
  child_doc.body.style.padding = '0';
  child_doc.body.style.overflow = 'hidden';
  child_doc.body.style.height = '100%';
  child_doc.body.style.width = '100%';
  child_doc.body.style.backgroundColor = window.getComputedStyle(document.body).backgroundColor || '#09090b';

  const root_container = child_doc.createElement('div');
  root_container.id = 'detached-root';
  root_container.style.height = '100%';
  root_container.style.width = '100%';
  root_container.style.display = 'flex';
  root_container.style.flexDirection = 'column';
  root_container.style.overflow = 'hidden';
  root_container.style.backfaceVisibility = 'hidden';
  child_doc.body.appendChild(root_container);
  return root_container;
}

function setup_tauri_bridge(
  url: string,
  window_name: string,
  title: string,
  width: number,
  height: number,
  on_close: () => void
) {
  let disposed = false;
  let closed_once = false;
  let native_window: import('@tauri-apps/api/webviewWindow').WebviewWindow | null = null;
  const unlisten_callbacks: Array<() => void> = [];

  const fire_close_once = () => {
    if (disposed || closed_once) return;
    closed_once = true;
    on_close();
  };

  const attach_tauri_listeners = async (
    win: import('@tauri-apps/api/webviewWindow').WebviewWindow
  ) => {
    if (typeof win.once !== 'function') return;
    unlisten_callbacks.push(await win.once('tauri://error', (e) => {
      console.error('[Portal_Window] Tauri window error:', e);
      fire_close_once();
    }));

    unlisten_callbacks.push(await win.once('tauri://close-requested', () => {
      fire_close_once();
    }));
  };

  const open_window = async () => {
    const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
    if (disposed) return;
    const existing = await WebviewWindow.getByLabel(window_name);
    if (disposed) return;
    if (existing) {
      native_window = existing;
      await attach_tauri_listeners(existing);
      await (existing as unknown as { setFocus?: () => Promise<void> }).setFocus?.();
      return;
    }
    const target_url = new URL(url, window.location.origin).href;
    const win = new WebviewWindow(window_name, {
      url: target_url,
      title: `${title} | ${i18n.t('common.detached_suffix')}`,
      width,
      height,
      resizable: true,
      decorations: true,
      transparent: false
    });
    native_window = win;
    await attach_tauri_listeners(win);
  };

  const opened = open_window().catch(error => {
    console.error('[Portal_Window] Failed to open Tauri window:', error);
    fire_close_once();
  });

  return () => {
    disposed = true;
    const close_window = async () => {
      await opened;
      for (const unlisten of unlisten_callbacks) unlisten();
      await native_window?.close();
    };
    close_window().catch(error => {
      console.error('[Portal_Window] Failed to close Tauri window:', error);
    });
  };
}

function Portal_Header({ title, on_recall }: { title: string; on_recall?: () => void }) {
  return (
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
        {on_recall && (
          <button
            onClick={on_recall}
            className="flex items-center gap-1.5 px-2 py-0.5 text-[10px] font-mono font-medium text-zinc-400 hover:text-zinc-100 bg-zinc-900/80 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 rounded transition-all active:scale-95 group"
            title={i18n.t('chat.restore_tooltip') || 'Recall / Dock Window'}
          >
            <Minimize2 size={11} className="group-hover:-rotate-45 transition-transform" />
            <span>RECALL</span>
          </button>
        )}
      </div>
    </header>
  );
}

function attach_window_lifecycle(target_win: Window, on_close: () => void) {
  target_win.addEventListener('pagehide', on_close);
  target_win.addEventListener('beforeunload', on_close);
  target_win.addEventListener('unload', on_close);
}

function detach_window_lifecycle(target_win: Window, on_close: () => void) {
  target_win.removeEventListener('pagehide', on_close);
  target_win.removeEventListener('beforeunload', on_close);
  target_win.removeEventListener('unload', on_close);
}

/**
 * Popups whose close was deferred by a cleanup. React StrictMode (and fast
 * unmount/remount cycles) run effect -> cleanup -> effect synchronously. The
 * first window.open() consumes the click's transient user activation, so a
 * second open is popup-blocked and returns null, which used to revert the
 * detach state ("screen flashes"). Deferring close lets the remount reclaim
 * the same window without calling window.open() again.
 */
const pending_popup_closes = new Map<string, { win: Window; timer: ReturnType<typeof setTimeout> }>();

function acquire_popup(window_name: string, features: string): Window | null {
  const pending = pending_popup_closes.get(window_name);
  if (pending) {
    clearTimeout(pending.timer);
    pending_popup_closes.delete(window_name);
    if (!pending.win.closed) return pending.win;
  }
  try {
    return window.open('', window_name, features);
  } catch (err) {
    console.error('[Portal_Window] Error opening popup:', err);
    return null;
  }
}

function release_popup(window_name: string, win: Window) {
  const timer = setTimeout(() => {
    pending_popup_closes.delete(window_name);
    win.close();
  }, 0);
  pending_popup_closes.set(window_name, { win, timer });
}

function setup_browser_portal(
  window_name: string,
  title: string,
  get_dimensions: () => { width: number; height: number },
  is_auto_scale: boolean,
  on_close: () => void,
  on_popup_block?: () => void,
  on_ready?: (root: HTMLDivElement, win: Window) => void
) {
  let disposed = false;
  let closed_once = false;
  const fire_close_once = () => {
    if (disposed || closed_once) return;
    closed_once = true;
    on_close();
  };

  const { width, height } = get_dimensions();
  const features = `width=${width},height=${height},left=100,top=100,resizable=yes,scrollbars=yes,status=no,menubar=no,toolbar=no,location=no,directories=no`;
  const win = acquire_popup(window_name, features);

  if (!win) {
    console.warn('[Portal_Window] Popup blocked or failed to open.');
    on_popup_block?.();
    fire_close_once();
    return () => {
      disposed = true;
    };
  }

  const child_doc = win.document;
  const heartbeat = setInterval(() => {
    if (win?.closed) {
      clearInterval(heartbeat);
      fire_close_once();
    }
  }, 500);

  const root = child_doc.getElementById('detached-root') as HTMLDivElement | null || init_child_document(child_doc, title);
  queueMicrotask(() => {
    if (!disposed) on_ready?.(root, win!);
  });
  sync_document_styles(child_doc);

  const observer = new MutationObserver(() => sync_document_styles(child_doc));
  observer.observe(document.head, { childList: true, subtree: true });

  const update_scale = () => {
    const dims = get_dimensions();
    apply_window_scale(win!, root, dims.width, dims.height);
  };
  if (is_auto_scale) {
    update_scale();
    win.addEventListener('resize', update_scale);
  }

  attach_window_lifecycle(win, fire_close_once);
  win.focus();

  return () => {
    disposed = true;
    clearInterval(heartbeat);
    observer.disconnect();
    if (is_auto_scale && win) win.removeEventListener('resize', update_scale);
    detach_window_lifecycle(win, fire_close_once);
    release_popup(window_name, win);
  };
}

function setExternalWindowTitle(win: Window | null, title: string) {
  if (win) {
    win.document.title = `${title} | ${i18n.t('common.detached_suffix')}`;
  }
}

function useExternalWindowSync(
  win_ref: React.MutableRefObject<Window | null>,
  width: number,
  height: number,
  title: string
) {
  useEffect(() => {
    if (win_ref.current && width && height) {
      try { win_ref.current.resizeTo(width, height); } catch { /* ignore */ }
    }
  }, [width, height, win_ref]);

  useEffect(() => {
    setExternalWindowTitle(win_ref.current, title);
  }, [title, win_ref]);
}

interface PortalWindowLifecycleProps {
  id: string;
  title: string;
  width: number;
  height: number;
  url?: string;
  is_auto_scale: boolean;
  close_fn: () => void;
  popup_block_fn?: () => void;
}

function usePortalWindowLifecycle({
  id,
  title,
  width,
  height,
  url,
  is_auto_scale,
  close_fn,
  popup_block_fn
}: PortalWindowLifecycleProps) {
  const [container, set_container] = useState<HTMLDivElement | null>(null);
  const external_win_ref = useRef<Window | null>(null);
  const on_close_ref = useRef(close_fn);
  const on_popup_block_ref = useRef(popup_block_fn);
  const title_ref = useRef(title);
  const width_ref = useRef(width);
  const height_ref = useRef(height);

  useLayoutEffect(() => {
    on_close_ref.current = close_fn;
    on_popup_block_ref.current = popup_block_fn;
    title_ref.current = title;
    width_ref.current = width;
    height_ref.current = height;
  });

  const window_name = useMemo(() => `tadpole-detached-${id}`, [id]);

  useEffect(() => {
    const is_tauri = typeof window !== 'undefined' && 
      (!!(window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ || 
       !!(window as unknown as { __TAURI__: unknown }).__TAURI__);
    
    if (is_tauri && url) {
      return setup_tauri_bridge(
        url,
        window_name,
        title_ref.current,
        width_ref.current,
        height_ref.current,
        () => on_close_ref.current()
      );
    }

    const cleanup = setup_browser_portal(
      window_name,
      title_ref.current,
      () => ({ width: width_ref.current, height: height_ref.current }),
      is_auto_scale,
      () => on_close_ref.current(),
      () => on_popup_block_ref.current?.(),
      (root, win) => {
        external_win_ref.current = win;
        set_container(root);
      }
    );

    return () => {
      cleanup();
      external_win_ref.current = null;
      set_container(null);
    };
  }, [window_name, url, is_auto_scale]);

  useExternalWindowSync(external_win_ref, width, height, title);

  return { container, on_close_ref };
}

/**
 * PortalWindow / Portal_Window
 * Renders its children into a detached browser window while maintaining
 * the React component tree and state of the parent instance.
 */
export const PortalWindow: React.FC<Portal_Window_Props> = (props) => {
  const { children, title, id, width = 1200, height = 800, url, auto_scale, autoScale } = props;
  const is_auto_scale = autoScale ?? auto_scale ?? true;
  const close_fn = props.onClose || props.on_close || (() => {});
  const popup_block_fn = props.onPopupBlock || props.on_popup_block;

  const { container, on_close_ref } = usePortalWindowLifecycle({
    id,
    title,
    width,
    height,
    url,
    is_auto_scale,
    close_fn,
    popup_block_fn
  });

  if (!container) return null;

  return createPortal(
    <div className="w-full h-full bg-[color:var(--color-background)] text-zinc-100 selection:bg-zinc-700/30 font-sans antialiased overflow-hidden flex flex-col">
      <Portal_Header title={title} on_recall={props.onClose || props.on_close ? () => on_close_ref.current() : undefined} />
      <div className="flex-1 relative overflow-hidden flex flex-col">
        {children}
      </div>
    </div>,
    container
  );
};

export const Portal_Window = PortalWindow;
