/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Template_Grid
 * - **Primary Entrypoints**: `Template_Grid`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
