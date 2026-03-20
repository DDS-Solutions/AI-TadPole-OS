import { X, ExternalLink, Minimize2 } from 'lucide-react';
import * as Icons from 'lucide-react';
import clsx from 'clsx';
import { type Tab, useTabStore } from '../../stores/tabStore';
import { Tooltip } from '../ui';

interface TabItemProps {
    tab: Tab;
    isActive: boolean;
}

export function TabItem({ tab, isActive }: TabItemProps) {
    const { setActiveTab, closeTab, toggleTabDetachment } = useTabStore();
    
    // Dynamically resolve icon if it exists
    const IconComponent = (Icons as unknown as Record<string, React.ComponentType<{ size?: number }>>)[tab.icon || 'LayoutDashboard'] ?? Icons.LayoutDashboard;

    const handleClose = (e: React.MouseEvent) => {
        e.stopPropagation();
        closeTab(tab.id);
    };

    const handleDetach = (e: React.MouseEvent) => {
        e.stopPropagation();
        toggleTabDetachment(tab.id);
    };

    return (
        <div
            onClick={() => setActiveTab(tab.id)}
            className={clsx(
                "group relative flex items-center gap-2 px-4 py-2 min-w-[120px] max-w-[200px] cursor-pointer transition-all duration-200 border-r border-zinc-900",
                isActive 
                    ? "bg-zinc-900 text-zinc-100" 
                    : "text-zinc-500 hover:bg-zinc-900/40 hover:text-zinc-300"
            )}
        >
            {/* Active Indicator Bar */}
            {isActive && (
                <div className="absolute top-0 left-0 right-0 h-[2px] bg-zinc-400" />
            )}

            <div className={clsx(
                "flex-shrink-0 transition-colors",
                isActive ? "text-zinc-300" : "text-zinc-600 group-hover:text-zinc-400"
            )}>
                <IconComponent size={14} />
            </div>

            <span className="text-xs font-medium truncate flex-1 tracking-tight">
                {tab.title}
            </span>

            <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-all">
                <Tooltip content={tab.isDetached ? "Re-attach Tab" : "Detach to New Window"} position="top">
                    <button
                        onClick={handleDetach}
                        className={clsx(
                            "p-0.5 rounded-md hover:bg-zinc-800 transition-all",
                            isActive ? "text-zinc-500 hover:text-zinc-200" : "text-zinc-700 hover:text-zinc-400"
                        )}
                    >
                        {tab.isDetached ? <Minimize2 size={12} /> : <ExternalLink size={12} />}
                    </button>
                </Tooltip>

                <button
                    onClick={handleClose}
                    className={clsx(
                        "p-0.5 rounded-md hover:bg-zinc-800 transition-all",
                        isActive ? "text-zinc-500 hover:text-zinc-200" : "text-zinc-700 hover:text-zinc-400"
                    )}
                >
                    <X size={12} />
                </button>
            </div>
        </div>
    );
}
