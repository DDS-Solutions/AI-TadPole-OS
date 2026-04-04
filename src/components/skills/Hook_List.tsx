import React from 'react';
import { Anchor, Plus } from 'lucide-react';
import { type Hook_Definition } from '../../stores/skill_store';
import { Hook_Card } from './Hook_Card';
import { Tw_Empty_State } from '../ui/Tw_Empty_State';
import { i18n } from '../../i18n';

interface Hook_List_Props {
    hooks: Hook_Definition[];
    on_edit: (hook: Hook_Definition) => void;
    on_delete: (name: string) => void;
    on_create: () => void;
    is_loading?: boolean;
}

export const Hook_List: React.FC<Hook_List_Props> = ({ hooks, on_edit, on_delete, on_create, is_loading }) => {
    if (hooks.length === 0) {
        return (
            <div className="py-20 animate-in fade-in zoom-in-95 duration-500">
                <Tw_Empty_State
                    icon={<Anchor size={32} className="text-zinc-700" />}
                    title={i18n.t('skills.hooks_empty_title')}
                    description={i18n.t('skills.hooks_empty_description')}
                    action={
                        <button
                            onClick={on_create}
                            className="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-sm font-bold transition-all shadow-lg shadow-blue-500/20"
                        >
                            <Plus size={16} />
                            {i18n.t('skills.add_hook_button')}
                        </button>
                    }
                />
            </div>
        );
    }

    return (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 animate-in fade-in slide-in-from-bottom-5">
            {hooks.map((hook) => (
                <Hook_Card
                    key={hook.name}
                    hook={hook}
                    on_edit={on_edit}
                    on_delete={on_delete}
                    is_loading={is_loading}
                />
            ))}
        </div>
    );
};
