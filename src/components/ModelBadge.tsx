import { ChevronDown } from 'lucide-react';
import { getModelColor } from '../utils/modelUtils';

/**
 * Props for the ModelBadge component.
 */
interface ModelBadgeProps {
    /** The name of the model to display (e.g., "GPT-5.2") */
    model: string;
    /** Whether this model is currently active/processing */
    isActive?: boolean;
    /** Optional click handler for interactivity */
    onClick?: () => void;
}



/**
 * A badge component that displays an AI model's name with provider-specific styling.
 * Color map covers all Feb 2026 model providers.
 */
export const ModelBadge = ({ model, isActive, onClick }: ModelBadgeProps) => {
    const colorClass = getModelColor(model);

    return (
        <button
            onClick={onClick}
            className={`
                text-[10px] px-1.5 py-px rounded border border-opacity-50 font-medium flex items-center gap-1 
                hover:brightness-110 transition-all ${colorClass} flex-shrink-0
                ${onClick ? 'cursor-pointer' : ''}
                ${isActive ? 'animate-pulse' : ''}
            `}
            style={isActive ? {
                boxShadow: `0 0 12px currentColor`,
                borderColor: 'currentColor',
                outline: '1px solid currentColor',
                outlineOffset: '-1px'
            } : {}}
        >
            {model}
            {onClick && <ChevronDown size={8} className="opacity-70" />}
        </button>
    );
};
