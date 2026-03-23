import { i18n } from '../../i18n';
import type { Agent } from '../../types';

interface NodeTaskBoxProps {
    agent: Agent;
}

export const NodeTaskBox: React.FC<NodeTaskBoxProps> = ({ agent }) => {
    const isActive = agent.activeMission || agent.status === 'active';

    return (
        <div className={`
            text-xs min-h-[42px] max-h-[80px] overflow-y-auto leading-tight p-2 rounded-lg border z-10 transition-colors custom-scrollbar
            ${isActive ? 'bg-black/40 text-zinc-300 border-zinc-800' : 'bg-black/20 text-zinc-500 border-zinc-800/50'}
        `}>
            <div className="break-words">
                {agent.currentTask || <span className="italic opacity-50 font-mono text-[10px]">{i18n.t('agent_card.label_idle_task')}</span>}
            </div>
        </div>
    );
};
