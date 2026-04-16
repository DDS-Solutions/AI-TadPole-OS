/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Individual operational context node with detachment logic. 
 * Controls the persistence lifecycle, external window spawning, and active indicator animations.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: External window spawn refusal (pop-up blocker), `Icons` resolution fallback to `LayoutDashboard`, or close button click-through to active tab selection.
 * - **Telemetry Link**: Search for `[Tab_Item]` or `toggle_tab_detachment` in UI logs.
 */

import React from 'react';
import {
    X,
    ExternalLink,
    Minimize2,
    LayoutDashboard,
    Users,
    MessagesSquare,
    Grid,
    Target,
    Cpu,
    Bot,
    Zap,
    Shield,
    Wrench,
    BarChart,
    Clock,
    Store,
    BookOpen,
    Settings,
    ShoppingBag,
    Lock,
    type LucideProps,
} from 'lucide-react';
import clsx from 'clsx';
import { type Tab, use_tab_store } from '../../stores/tab_store';
import { Tooltip } from '../ui';

interface Tab_Item_Props {
    tab: Tab;
    is_active: boolean;
}

const TAB_ICONS: Record<string, React.ComponentType<LucideProps>> = {
    LayoutDashboard,
    Users,
    MessagesSquare,
    Grid,
    Target,
    Cpu,
    Bot,
    Zap,
    Shield,
    Wrench,
    BarChart,
    Clock,
    Store,
    BookOpen,
    Settings,
    ShoppingBag,
    Lock,
};

/**
 * Tab_Item
 * Individual tab component with persistence and detachment logic.
 * Refactored for strict snake_case compliance.
 */
export function Tab_Item({ tab, is_active }: Tab_Item_Props) {
    const { set_active_tab, close_tab, toggle_tab_detachment } = use_tab_store();
    
    // Dynamically resolve icon if it exists
    const Icon_Component = TAB_ICONS[tab.icon || 'LayoutDashboard'] ?? LayoutDashboard;

    const handle_close = (e: React.MouseEvent) => {
        e.stopPropagation();
        close_tab(tab.id);
    };

    const handle_detach = (e: React.MouseEvent) => {
        e.stopPropagation();
        toggle_tab_detachment(tab.id);
    };

    return (
        <div
            role="tab"
            aria-selected={is_active}
            tabIndex={0}
            onClick={() => set_active_tab(tab.id)}
            onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    set_active_tab(tab.id);
                }
            }}
            className={clsx(
                "group relative flex items-center gap-2 px-4 py-2 min-w-[120px] max-w-[200px] cursor-pointer transition-all duration-200 border-r border-zinc-900",
                is_active 
                    ? "bg-zinc-900 text-zinc-100" 
                    : "text-zinc-500 hover:bg-zinc-900/40 hover:text-zinc-300"
            )}
        >
            {/* Active Indicator Bar */}
            {is_active && (
                <div className="absolute top-0 left-0 right-0 h-[2px] bg-zinc-400" />
            )}

            <div className={clsx(
                "flex-shrink-0 transition-colors",
                is_active ? "text-zinc-300" : "text-zinc-600 group-hover:text-zinc-400"
            )}>
                <Icon_Component size={14} />
            </div>

            <span className="text-xs font-medium truncate flex-1 tracking-tight">
                {tab.title}
            </span>

            <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-all">
                <Tooltip content={tab.is_detached ? "Re-attach Tab" : "Detach to New Window"} position="top">
                    <button
                        onClick={handle_detach}
                        className={clsx(
                            "p-0.5 rounded-md hover:bg-zinc-800 transition-all",
                            is_active ? "text-zinc-500 hover:text-zinc-200" : "text-zinc-700 hover:text-zinc-400"
                        )}
                    >
                        {tab.is_detached ? <Minimize2 size={12} /> : <ExternalLink size={12} />}
                    </button>
                </Tooltip>

                <button
                    onClick={handle_close}
                    className={clsx(
                        "p-0.5 rounded-md hover:bg-zinc-800 transition-all",
                        is_active ? "text-zinc-500 hover:text-zinc-200" : "text-zinc-700 hover:text-zinc-400"
                    )}
                >
                    <X size={12} />
                </button>
            </div>
        </div>
    );
}

