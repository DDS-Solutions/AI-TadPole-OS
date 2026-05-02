/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: High-fidelity indicator for neural model identity. 
 * Color-codes badges based on provider (OpenAI, Gemini, Anthropic) and visualizes active processing states with pulse animations.
 * Features **IMR-01 (Intelligent Model Registry)** visualization: renders capability icons (Eye/Wrench/Brain) based on backend inferred metadata.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Provider color map lookup failure (gray fallback), pulse animation stuttering during high-frequency telemetry bursts, or capability icons missing due to null metadata.
 * - **Telemetry Link**: Search for `[Model_Badge]` or `active_model_pulse` in UI tracing.
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
            <div className="flex items-center gap-1">
                {capabilities?.supports_vision && <Eye size={8} className="text-zinc-400" />}
                {capabilities?.supports_tools && <Wrench size={8} className="text-zinc-400" />}
                {capabilities?.supports_reasoning && <Brain size={8} className="text-zinc-400" />}
                <span className="truncate max-w-[80px]">{model}</span>
                {on_click && <ChevronDown size={8} className="opacity-70" />}
            </div>
        </button>
    );
};


// Metadata: [Model_Badge]
