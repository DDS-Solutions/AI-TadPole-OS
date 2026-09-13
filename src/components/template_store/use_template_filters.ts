/**
 * @docs ARCHITECTURE:Hooks
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / use_template_filters
 * - **Primary Entrypoints**: `useTemplateFilters`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
