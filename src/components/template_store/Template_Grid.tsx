/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Template_Grid]` in observability traces.
 */

import type { Template } from './types';
import { Template_Card } from './Template_Card';
import { Loading_State } from './states/Loading_State';
import { Error_State } from './states/Error_State';
import { Empty_State } from './states/Empty_State';

interface TemplateGridProps {
    templates: Template[];
    isLoading: boolean;
    error: string | null;
    isInstalling: string | null;
    onPreview: (template: Template) => void;
}

export function Template_Grid({
    templates,
    isLoading,
    error,
    isInstalling,
    onPreview
}: TemplateGridProps) {
    if (isLoading) {
        return <Loading_State />;
    }

    if (error) {
        return <Error_State error={error} />;
    }

    return (
        <div className="pb-8">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {templates.map((template) => (
                    <Template_Card
                        key={template.id}
                        template={template}
                        isInstalling={isInstalling === template.id}
                        onPreview={onPreview}
                    />
                ))}
            </div>
            {templates.length === 0 && <Empty_State />}
        </div>
    );
}

// Metadata: [Template_Grid]
