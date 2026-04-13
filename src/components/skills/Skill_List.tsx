/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: High-density grid for manual agent capabilities. 
 * Orchestrates categorized skill rendering and facilitates empty-state transitions for the Capability Forge.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Grid re-order flicker on skill deletion, or empty state occlusion.
 * - **Telemetry Link**: Search for `[Skill_List]` or `load_skill_catalog` in UI tracing.
 */

import React from 'react';
import { Activity, Plus } from 'lucide-react';
import { Tooltip, Tw_Empty_State } from '../ui';
import { i18n } from '../../i18n';
import type { Skill_Definition } from '../../stores/skill_store';
import { Skill_Card } from './Skill_Card';

interface Skill_List_Props {
    skills: Skill_Definition[];
    search_query: string;
    active_category: 'user' | 'ai';
    on_new_skill: () => void;
    on_edit_skill: (skill: Skill_Definition) => void;
    on_assign_skill: (name: string) => void;
    on_delete_skill: (name: string) => void;
}

export const Skill_List: React.FC<Skill_List_Props> = ({
    skills,
    search_query,
    active_category,
    on_new_skill,
    on_edit_skill,
    on_assign_skill,
    on_delete_skill
}) => {
    const filtered_skills = skills.filter(s => 
        s.category === active_category && 
        (s.name.toLowerCase().includes(search_query.toLowerCase()) || 
         s.description?.toLowerCase().includes(search_query.toLowerCase()))
    );

    return (
        <div className="space-y-6">
            <div className="flex justify-between items-center mb-6 sticky top-0 bg-zinc-950/80 backdrop-blur-md pt-2 pb-3 border-b border-zinc-800/50 z-20">
                <h3 className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                    <Activity size={12} className="text-blue-500" /> {i18n.t('skills.header_execution')}
                </h3>
                <Tooltip content={i18n.t('skills.tooltip_new_skill')} position="bottom">
                    <button
                        onClick={on_new_skill}
                        className="bg-blue-600 hover:bg-blue-500 text-white px-3 py-1.5 rounded-lg text-xs font-bold flex items-center gap-2 transition-colors shadow-[0_0_15px_rgba(37,99,235,0.3)]"
                    >
                        <Plus className="w-3.5 h-3.5" /> {i18n.t('skills.btn_new_skill')}
                    </button>
                </Tooltip>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 relative z-10">
                {filtered_skills.map(skill => (
                    <Skill_Card 
                        key={skill.name} 
                        skill={skill} 
                        on_edit={on_edit_skill} 
                        on_assign={on_assign_skill} 
                        on_delete={on_delete_skill} 
                    />
                ))}
                {filtered_skills.length === 0 && (
                    <div className="col-span-full">
                        <Tw_Empty_State 
                            title={i18n.t('skills.empty_skills_title')} 
                            description={i18n.t('skills.empty_skills_desc')} 
                        />
                    </div>
                )}
            </div>
        </div>
    );
};

