/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Repository_Actions
 * - **Primary Entrypoints**: `Repository_Actions`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { RefreshCw, ExternalLink } from 'lucide-react';
import { i18n } from '../../i18n';

interface RepositoryActionsProps {
    isLoading: boolean;
    onRefresh: () => void;
}

export function Repository_Actions({ isLoading, onRefresh }: RepositoryActionsProps) {
    return (
        <div className="flex items-center gap-2 w-full sm:w-auto">
            <button
                onClick={onRefresh}
                disabled={isLoading}
                className="flex items-center justify-center gap-2 px-4 py-2 bg-[color:var(--color-surface)]/80 hover:bg-zinc-800 border border-[color:var(--color-border)] hover:border-zinc-700 text-zinc-300 hover:text-white rounded-xl text-xs font-bold transition-all duration-200 active:scale-95 disabled:opacity-50 disabled:cursor-wait"
            >
                <RefreshCw size={14} className={isLoading ? 'animate-spin' : ''} />
                <span>{i18n.t('template_store.btn_scan_repo')}</span>
            </button>
            <a
                href="https://dds-solutions.github.io/AI-Tadpole-OS-Industry-Templates/"
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center justify-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-500 text-white border border-green-500 rounded-xl text-xs font-bold transition-all duration-200 active:scale-95 shadow-lg shadow-green-500/20"
            >
                <ExternalLink size={14} />
                <span>{i18n.t('template_store.btn_make_templates')}</span>
            </a>
        </div>
    );
}
