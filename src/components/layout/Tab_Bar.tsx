import { use_tab_store } from '../../stores/tab_store';
import { Tab_Item } from './Tab_Item';


export function Tab_Bar() {
    const { tabs, active_tab_id } = use_tab_store();

    if (tabs.length === 0) return null;

    return (
        <div className="flex bg-zinc-950 border-b border-zinc-900 h-10 overflow-x-auto no-scrollbar select-none items-stretch">
            {tabs.map((tab) => (
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
