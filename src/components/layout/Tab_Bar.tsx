/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Persistent navigation layer for active operational contexts. 
 * Orchestrates tab switching, horizontal overflow scrolling, and `tab_store` synchronization.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Tab sync issue (zombie tabs) if `tab_store` hydration is interrupted, or horizontal scroll blocking on touch devices.
 * - **Telemetry Link**: Search for `[Tab_Bar]` or `set_active_tab` in browser tracing.
 */

import { use_tab_store } from '../../stores/tab_store';
import { Tab_Item } from './Tab_Item';


export function Tab_Bar() {
    const { tabs, active_tab_id } = use_tab_store();

    const safe_tabs = tabs || [];
    if (safe_tabs.length === 0) return null;

    return (
        <div className="flex bg-zinc-950 border-b border-zinc-900 h-10 overflow-x-auto no-scrollbar select-none items-stretch">
            {safe_tabs.map((tab) => (
                <Tab_Item 
                    key={tab.id} 
                    tab={tab} 
                    is_active={tab.id === active_tab_id} 
                />
            ))}
            
            {/* Filler space */}
            <div className="flex-1 border-b-zinc-900" />
        </div>
    );
}

// Metadata: [Tab_Bar]
