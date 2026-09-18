/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Layout / Tab_Bar
 * - **Primary Entrypoints**: `Tab_Bar`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { use_tab_store } from '../../stores/tab_store';
import { Tab_Item } from './Tab_Item';


export function Tab_Bar() {
    const { tabs, active_tab_id } = use_tab_store();

    const safe_tabs = tabs || [];
    if (safe_tabs.length === 0) return null;

    return (
        <div className="flex bg-[color:var(--color-background)] border-b border-[color:var(--color-surface)] h-10 overflow-x-auto no-scrollbar select-none items-stretch">
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
