import { ChevronDown } from 'lucide-react';
import { get_model_color } from '../utils/model_utils';

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
}

/**
 * Model_Badge
 * A badge component that displays an AI model's name with provider-specific styling.
 * Color map covers all Feb 2026 model providers.
 * Refactored for strict snake_case compliance for backend parity.
 */
export const Model_Badge = ({ model, is_active, on_click }: Model_Badge_Props) => {
    const color_class = get_model_color(model);

    return (
        <button
            onClick={on_click}
            className={`
                text-[10px] px-1.5 py-px rounded border border-opacity-50 font-medium flex items-center gap-1 
                hover:brightness-110 transition-all ${color_class} flex-shrink-0
                ${on_click ? 'cursor-pointer' : ''}
                ${is_active ? 'animate-pulse' : ''}
            `}
            style={is_active ? {
                boxShadow: `0 0 12px currentColor`,
                borderColor: 'currentColor',
                outline: '1px solid currentColor',
                outlineOffset: '-1px'
            } : {}}
        >
            {model}
            {on_click && <ChevronDown size={8} className="opacity-70" />}
        </button>
    );
};
