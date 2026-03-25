import React from 'react';
import { Activity, Plus } from 'lucide-react';
import { Tooltip, TwEmptyState } from '../ui';
import { i18n } from '../../i18n';
import type { SkillDefinition } from '../../stores/skillStore';
import { SkillCard } from './SkillCard';

interface SkillListProps {
    skills: SkillDefinition[];
    searchQuery: string;
    activeCategory: 'user' | 'ai';
    onNewSkill: () => void;
    onEditSkill: (skill: SkillDefinition) => void;
    onAssignSkill: (name: string) => void;
    onDeleteSkill: (name: string) => void;
}

export const SkillList: React.FC<SkillListProps> = ({
    skills,
    searchQuery,
    activeCategory,
    onNewSkill,
    onEditSkill,
    onAssignSkill,
    onDeleteSkill
}) => {
    const filteredSkills = skills.filter(s => 
        s.category === activeCategory && 
        (s.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
         s.description?.toLowerCase().includes(searchQuery.toLowerCase()))
    );

    return (
        <div className="space-y-6">
            <div className="flex justify-between items-center mb-6 sticky top-0 bg-zinc-950/80 backdrop-blur-md pt-2 pb-3 border-b border-zinc-800/50 z-20">
                <h3 className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                    <Activity size={12} className="text-blue-500" /> {i18n.t('skills.header_execution')}
                </h3>
                <Tooltip content={i18n.t('skills.tooltip_new_skill')} position="bottom">
                    <button
                        onClick={onNewSkill}
                        className="bg-blue-600 hover:bg-blue-500 text-white px-3 py-1.5 rounded-lg text-xs font-bold flex items-center gap-2 transition-colors shadow-[0_0_15px_rgba(37,99,235,0.3)]"
                    >
                        <Plus className="w-3.5 h-3.5" /> {i18n.t('skills.btn_new_skill')}
                    </button>
                </Tooltip>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 relative z-10">
                {filteredSkills.map(skill => (
                    <SkillCard 
                        key={skill.name} 
                        skill={skill} 
                        onEdit={onEditSkill} 
                        onAssign={onAssignSkill} 
                        onDelete={onDeleteSkill} 
                    />
                ))}
                {filteredSkills.length === 0 && (
                    <div className="col-span-full">
                        <TwEmptyState 
                            title={i18n.t('skills.empty_skills_title')} 
                            description={i18n.t('skills.empty_skills_desc')} 
                        />
                    </div>
                )}
            </div>
        </div>
    );
};
