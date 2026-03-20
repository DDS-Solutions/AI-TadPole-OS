import { useTabStore } from '../../stores/tabStore';
import { TabItem } from './TabItem';


export function TabBar() {
    const { tabs, activeTabId } = useTabStore();

    if (tabs.length === 0) return null;

    return (
        <div className="flex bg-zinc-950 border-b border-zinc-900 h-10 overflow-x-auto no-scrollbar select-none items-stretch">
            {tabs.map((tab) => (
                <TabItem 
                    key={tab.id} 
                    tab={tab} 
                    isActive={tab.id === activeTabId} 
                />
            ))}
            
            {/* Filler space */}
            <div className="flex-1 border-b-zinc-900" />
        </div>
    );
}
