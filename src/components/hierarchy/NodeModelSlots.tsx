import React from 'react';
import { i18n } from '../../i18n';
import { Tooltip } from '../ui';
import { ModelBadge } from '../ModelBadge';
import { useDropdownStore } from '../../stores/dropdownStore';
import { useModelStore } from '../../stores/modelStore';
import type { Agent } from '../../types';

interface NodeModelSlotsProps {
    agent: Agent;
    onModelChange?: (agentId: string, newModel: string) => void;
    onModel2Change?: (agentId: string, newModel: string) => void;
    onModel3Change?: (agentId: string, newModel: string) => void;
    onUpdate?: (agentId: string, updates: Partial<Agent>) => void;
}

export const NodeModelSlots: React.FC<NodeModelSlotsProps> = ({
    agent,
    onModelChange,
    onModel2Change,
    onModel3Change,
    onUpdate
}) => {
    const toggle = useDropdownStore(s => s.toggle);
    const close = useDropdownStore(s => s.close);
    const isModel1Open = useDropdownStore(s => s.openId === agent.id && s.openType === 'model');
    const isModel2Open = useDropdownStore(s => s.openId === agent.id && s.openType === 'model2');
    const isModel3Open = useDropdownStore(s => s.openId === agent.id && s.openType === 'model3');

    const availableModels = useModelStore(s => s.models);
    const sortedModelNames = React.useMemo(() =>
        Array.from(new Set(availableModels.map(m => m.name))).sort(),
        [availableModels]);

    const renderSlot = (slotIdx: 1 | 2 | 3, model: string | undefined, isOpen: boolean, onModelUpdate?: (id: string, m: string) => void) => {
        const isActiveSlot = agent.activeModelSlot === slotIdx || (slotIdx === 1 && !agent.activeModelSlot && agent.status !== 'idle' && agent.status !== 'offline');

        const ledColor =
            slotIdx === 1 ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]' :
                slotIdx === 2 ? 'bg-blue-500 shadow-[0_0_8px_rgba(59,130,246,0.5)]' :
                    'bg-amber-500 shadow-[0_0_8px_rgba(245,158,11,0.5)]';

        return (
            <div className="relative" key={`slot-${slotIdx}`} onClick={(e) => e.stopPropagation()}>
                <Tooltip content={i18n.t(`agent_card.tooltip_activate_${slotIdx === 1 ? 'primary' : slotIdx === 2 ? 'secondary' : 'tertiary'}`)} position="top">
                    <button
                        onClick={(e) => {
                            e.stopPropagation();
                            if (agent.activeModelSlot !== slotIdx) {
                                onUpdate?.(agent.id, { activeModelSlot: slotIdx });
                            }
                        }}
                        className="absolute -top-7 left-1/2 -translate-x-1/2 w-6 h-6 flex items-center justify-center cursor-pointer z-30 group/led"
                    >
                        <div className={`
                            w-1.5 h-1.5 rounded-full transition-all duration-300
                            ${isActiveSlot ? `${ledColor} scale-110` : 'bg-zinc-800 group-hover/led:bg-zinc-700'}
                        `} />
                    </button>
                </Tooltip>
                <ModelBadge
                    model={model || (slotIdx === 1 ? i18n.t('agent_card.label_unknown_model') : i18n.t('agent_card.label_add_model'))}
                    isActive={isActiveSlot}
                    onClick={() => toggle(agent.id, slotIdx === 1 ? 'model' : slotIdx === 2 ? 'model2' : 'model3')}
                />
                {isOpen && (
                    <div className="absolute top-full left-0 mt-1 w-48 bg-zinc-900 border border-zinc-700 rounded-lg shadow-xl z-50 py-1 max-h-60 overflow-y-auto custom-scrollbar">
                        {sortedModelNames.map((m) => (
                            <button key={m} onClick={() => {
                                onModelUpdate?.(agent.id, m);
                                close();
                            }}
                                className={`w-full text-left px-3 py-1.5 text-xs hover:bg-zinc-800 transition-colors ${model === m ? 'text-emerald-400 font-bold bg-emerald-900/10' : 'text-zinc-300'}`}>
                                {m}
                            </button>
                        ))}
                    </div>
                )}
            </div>
        );
    };

    return (
        <div className={`flex flex-col gap-1.5 border-t border-zinc-800 pt-2 relative ${isModel1Open || isModel2Open || isModel3Open ? 'z-50' : 'z-20'}`}>
            <div className="flex items-center gap-1.5 overflow-visible">
                {renderSlot(1, agent.model, isModel1Open, onModelChange)}
                {renderSlot(2, agent.model2, isModel2Open, onModel2Change)}
                {renderSlot(3, agent.model3, isModel3Open, onModel3Change)}
            </div>
        </div>
    );
};
