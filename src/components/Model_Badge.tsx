/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / General / Model_Badge
 * - **Primary Entrypoints**: `Model_Badge`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { ChevronDown, Eye, Wrench, Brain } from 'lucide-react';
import { get_model_color } from '../utils/model_utils';
import { i18n } from '../i18n';

/**
 * Model_Badge_Props
 * Defines the interface for the Model_Badge component.
 */
interface Model_Badge_Props {
    /** The name of the model to display (e.g., "GPT-5.2") */
    model: string;
    /** Whether this model is currently active/processing */
    is_active?: boolean;
    /** Optional click handler for interactivity */
    on_click?: () => void;
    /** Optional capability metadata */
    capabilities?: {
        supports_tools?: boolean;
        supports_vision?: boolean;
        supports_reasoning?: boolean;
    };
}

/**
 * Model_Badge
 * A badge component that displays an AI model's name with provider-specific styling.
 * Color map covers all Feb 2026 model providers.
 * Refactored for strict snake_case compliance for backend parity.
 */
export const Model_Badge = ({ model, is_active, on_click, capabilities }: Model_Badge_Props) => {
    const color_class = get_model_color(model);

    return (
        <button
            onClick={on_click}
            aria-label={i18n.t('agent_manager.aria_model_selector', { model })}
            className={`
                text-[10px] px-1.5 py-px rounded border border-opacity-50 font-medium flex items-center gap-1.5 
                hover:brightness-110 transition-all ${color_class} flex-shrink-0
                ${on_click ? 'cursor-pointer' : ''}
                ${is_active ? 'animate-pulse' : ''}
            `}
            style={is_active ? {
                boxShadow: `0 0 12px currentColor, inset 0 0 0 1px currentColor`,
                borderColor: 'currentColor'
            } : {}}
        >
            <div className="flex items-center gap-1 min-w-0">
                {capabilities?.supports_vision && <Eye size={8} className="text-zinc-400 shrink-0" />}
                {capabilities?.supports_tools && <Wrench size={8} className="text-zinc-400 shrink-0" />}
                {capabilities?.supports_reasoning && <Brain size={8} className="text-zinc-400 shrink-0" />}
                <span className="truncate max-w-[180px]">{model}</span>
                {on_click && <ChevronDown size={8} className="opacity-70 shrink-0" />}
            </div>
        </button>
    );
};
