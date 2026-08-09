/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Template_Filters]` in observability traces.
 */

import { Search_Bar } from './Search_Bar';
import { Industry_Filters } from './Industry_Filters';
import { Size_Filters } from './Size_Filters';

interface TemplateFiltersProps {
    searchQuery: string;
    setSearchQuery: (val: string) => void;
    selectedIndustry: string;
    setSelectedIndustry: (val: string) => void;
    selectedCompanySize: string;
    setSelectedCompanySize: (val: string) => void;
    industries: string[];
    onImportDownloadedSwarm?: (file: File) => void;
}

export function Template_Filters({
    searchQuery,
    setSearchQuery,
    selectedIndustry,
    setSelectedIndustry,
    selectedCompanySize,
    setSelectedCompanySize,
    industries,
    onImportDownloadedSwarm
}: TemplateFiltersProps) {
    return (
        <div className="flex flex-col gap-4">
            <Search_Bar value={searchQuery} onChange={setSearchQuery} />
            <Industry_Filters
                industries={industries}
                selected={selectedIndustry}
                onSelect={setSelectedIndustry}
            />
            <Size_Filters
                selected={selectedCompanySize}
                onSelect={setSelectedCompanySize}
                onImportDownloadedSwarm={onImportDownloadedSwarm}
            />
        </div>
    );
}

// Metadata: [Template_Filters]
