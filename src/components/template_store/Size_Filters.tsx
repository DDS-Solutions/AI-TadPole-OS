/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Size_Filters]` in observability traces.
 */

import { FolderDown } from 'lucide-react';
import { COMPANY_SIZES } from './constants';
import { i18n } from '../../i18n';

interface SizeFiltersProps {
    selected: string;
    onSelect: (size: string) => void;
    onImportDownloadedSwarm?: (file: File) => void;
}

export function Size_Filters({ selected, onSelect, onImportDownloadedSwarm }: SizeFiltersProps) {
    return (
        <div data-testid="size-filters" className="flex items-center gap-2 overflow-x-auto pb-2 custom-scrollbar w-full">
            <span className="sovereign-header-text mr-2 flex-shrink-0">
                {i18n.t('template_store.label_size')}
            </span>
            {COMPANY_SIZES.map((size) => (
                <button
                    key={size}
                    onClick={() => onSelect(size)}
                    className={`px-3 py-1 rounded-md text-[10px] font-bold whitespace-nowrap transition-all ${
                        selected === size
                            ? 'bg-emerald-600 text-white shadow-lg shadow-emerald-500/20'
                            : 'bg-[color:var(--color-surface)] text-zinc-500 hover:bg-zinc-800 border border-[color:var(--color-border)]'
                    }`}
                >
                    {size}
                    {size !== 'All' ? ` ${i18n.t('template_store.employees')}` : ''}
                </button>
            ))}

            <label className="flex items-center gap-1.5 px-3 py-1 rounded-md text-[10px] font-bold whitespace-nowrap transition-all bg-cyan-600/20 hover:bg-cyan-600/30 hover:border-cyan-400 text-cyan-400 border border-cyan-500/40 cursor-pointer shadow-sm active:scale-95 ml-2">
                <FolderDown size={12} className="text-cyan-400" />
                <span>{i18n.t('template_store.btn_downloaded_swarms', { defaultValue: 'Downloaded Swarms' })}</span>
                <input
                    type="file"
                    accept=".json,.zip,.tadpole"
                    className="hidden"
                    onChange={(e) => {
                        const file = e.target.files?.[0];
                        if (file && onImportDownloadedSwarm) {
                            onImportDownloadedSwarm(file);
                            e.target.value = '';
                        }
                    }}
                />
            </label>
        </div>
    );
}

// Metadata: [Size_Filters]

