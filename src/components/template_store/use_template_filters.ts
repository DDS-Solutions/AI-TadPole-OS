/**
 * @docs ARCHITECTURE:Hooks
 * 
 * ### AI Assist Note
 * **Custom Hook**: Encapsulates search and category filtration with memoization.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: N/A
 * - **Telemetry Link**: Search `[Template_Store]` in telemetry traces.
 */

import { useState, useMemo } from 'react';
import type { Template } from './types';

export function useTemplateFilters(templates: Template[]) {
    const [searchQuery, setSearchQuery] = useState('');
    const [selectedIndustry, setSelectedIndustry] = useState('All');
    const [selectedCompanySize, setSelectedCompanySize] = useState('All');

    const industries = useMemo(() => {
        const unique = Array.from(new Set(templates.map((t) => t.industry)));
        return ['All', ...unique.sort((a, b) => a.localeCompare(b))];
    }, [templates]);

    const filteredTemplates = useMemo(() => {
        const query = searchQuery.toLowerCase().trim();
        return templates.filter((t) => {
            const matchSearch =
                !query ||
                t.name.toLowerCase().includes(query) ||
                t.description.toLowerCase().includes(query);
            const matchIndustry =
                selectedIndustry === 'All' ||
                t.industry.toLowerCase() === selectedIndustry.toLowerCase();
            const matchSize =
                selectedCompanySize === 'All' ||
                String(t.company_size) === selectedCompanySize;

            return matchSearch && matchIndustry && matchSize;
        });
    }, [templates, searchQuery, selectedIndustry, selectedCompanySize]);

    return {
        searchQuery,
        setSearchQuery,
        selectedIndustry,
        setSelectedIndustry,
        selectedCompanySize,
        setSelectedCompanySize,
        industries,
        filteredTemplates
    };
}

// Metadata: [Template_Store]


// [Template_Store]
