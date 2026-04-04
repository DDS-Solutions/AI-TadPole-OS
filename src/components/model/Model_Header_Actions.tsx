import React from 'react';
import { Lock, Activity } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

interface Model_Header_Actions_Props {
    on_lock: () => void;
    providers_count: number;
}

export const Model_Header_Actions: React.FC<Model_Header_Actions_Props> = ({
    on_lock,
    providers_count
}) => {
    return (
        <div className="flex items-center gap-4">
            <div className="flex items-center gap-2 text-[11px] font-bold text-zinc-500 font-mono tracking-widest uppercase">
                <Activity size={12} className="text-emerald-500" />
                {i18n.t('model_manager.header.status', { count: providers_count })}
            </div>
            <div className="h-4 w-px bg-zinc-800" />
            <Tooltip content={i18n.t('model_manager.header.tooltip_lock')} position="left">
                <button
                    onClick={on_lock}
                    className="flex items-center gap-2 px-3 py-1.5 bg-zinc-900 border border-zinc-800 rounded-lg text-[11px] font-bold text-zinc-500 hover:text-emerald-400 hover:border-emerald-500/30 transition-all uppercase tracking-widest"
                >
                    <Lock size={12} /> {i18n.t('model_manager.header.btn_lock')}
                </button>
            </Tooltip>
        </div>
    );
};
