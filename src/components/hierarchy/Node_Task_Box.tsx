import { motion, AnimatePresence } from 'framer-motion';
import { i18n } from '../../i18n';
import type { Agent } from '../../types';

interface Node_Task_Box_Props {
    agent: Agent;
}

export const Node_Task_Box: React.FC<Node_Task_Box_Props> = ({ agent }) => {
    const is_active = agent.active_mission || agent.status === 'active';

    return (
        <motion.div
            initial={false}
            animate={{
                borderColor: is_active ? 'rgba(56, 189, 248, 0.4)' : 'rgba(39, 39, 42, 0.5)',
                backgroundColor: is_active ? 'rgba(0, 0, 0, 0.4)' : 'rgba(0, 0, 0, 0.2)'
            }}
            className={`
                text-[11px] min-h-[42px] max-h-[80px] overflow-y-auto leading-tight p-2.5 rounded-lg border z-10 transition-colors custom-scrollbar relative
            `}
        >
            <AnimatePresence mode="wait">
                <motion.div
                    key={agent.current_task || 'idle'}
                    initial={{ opacity: 0, y: 5 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -5 }}
                    transition={{ duration: 0.2 }}
                    className="break-words"
                >
                    {agent.current_task ? (
                        <div className="flex flex-col gap-1">
                            <span className="text-zinc-300">{agent.current_task}</span>
                            <motion.div
                                animate={{ opacity: [0.3, 0.6, 0.3] }}
                                transition={{ repeat: Infinity, duration: 2 }}
                                className="h-0.5 w-full bg-blue-500/30 rounded-full"
                            />
                        </div>
                    ) : agent.status === 'suspended' ? (
                        <div className="flex items-center justify-center gap-2 italic font-mono text-[9px] text-rose-400 h-full min-h-[42px]">
                            <div className="w-1.5 h-1.5 rounded-full bg-rose-500 shadow-[0_0_8px_rgba(251,113,133,0.8)]" />
                            {i18n.t('agent_card.label_suspended_task')}
                        </div>
                    ) : (
                        <div className="flex items-center justify-center gap-2 italic font-mono text-[9px] text-zinc-200 h-full min-h-[42px]">
                            <div className="w-1.5 h-1.5 rounded-full bg-zinc-500 animate-pulse" />
                            {i18n.t('agent_card.label_idle_task')}
                        </div>
                    )}
                </motion.div>
            </AnimatePresence>
        </motion.div>
    );
};
