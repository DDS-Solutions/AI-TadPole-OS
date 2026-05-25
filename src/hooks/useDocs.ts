/**
 * @docs ARCHITECTURE:UI-Hooks
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[useDocs]` in observability traces.
 */

import { useState, useEffect, useMemo } from 'react';
import { tadpole_os_service } from '../services/tadpoleos_service';

export type DocMetadata = {
    category: string;
    name: string;
    title: string;
};

export type TOCItem = {
    id: string;
    text: string;
    level: number;
};

export function useDocs() {
    const [activeTab, set_active_tab] = useState<'knowledge' | 'manual'>('knowledge');
    const [searchTerm, setSearchTerm] = useState('');
    const [docs, setDocs] = useState<DocMetadata[]>([]);
    const [selectedDoc, setSelectedDoc] = useState<DocMetadata | null>(null);
    const [content, setContent] = useState<string>('');
    const [isLoading, setIsLoading] = useState(true);
    const [expandedCategories, setExpandedCategories] = useState<Record<string, boolean>>({});

    // ── Fetch Document List ────────────────────────────
    useEffect(() => {
        const fetchDocs = async () => {
            try {
                const docList = await tadpole_os_service.get_knowledge_docs();
                setDocs(docList);

                // Initialize categories as expanded
                const cats = Array.from(new Set(docList.map((d: DocMetadata) => d.category)));
                const expanded: Record<string, boolean> = {};
                cats.forEach(c => expanded[c] = true);
                setExpandedCategories(expanded);

                // Auto-select architecture doc or first available
                if (docList.length > 0) {
                    const initial = docList.find((d: DocMetadata) => d.name.includes('architecture')) || docList[0];
                    setSelectedDoc(initial);
                }
            } catch (error) {
                console.error('Failed to fetch doc list:', error);
            } finally {
                setIsLoading(false);
            }
        };
        fetchDocs();
    }, []);

    // ── Fetch Content ──────────────────────────────────
    useEffect(() => {
        const fetchContent = async () => {
            setIsLoading(true);
            try {
                let rawData = '';
                if (activeTab === 'manual') {
                    rawData = await tadpole_os_service.get_operations_manual();
                } else if (selectedDoc) {
                    rawData = await tadpole_os_service.get_knowledge_doc(selectedDoc.category, selectedDoc.name);
                }

                // Strip Frontmatter
                const frontmatterRegex = /^---\n([\s\S]*?)\n---\n([\s\S]*)$/;
                const match = rawData.match(frontmatterRegex);
                setContent(match ? match[2].trim() : rawData.trim());
            } catch (error) {
                const err = error as Error;
                console.error('Failed to fetch content:', err);
                setContent(`# Connection Failed\n${err?.message || String(err)}`);
            } finally {
                setIsLoading(false);
            }
        };
        fetchContent();
    }, [activeTab, selectedDoc]);

    // ── Grouped Docs (Filtered) ───────────────────────
    const groupedDocs = useMemo(() => {
        const groups: Record<string, DocMetadata[]> = {};
        const filtered = searchTerm
            ? docs.filter((d: DocMetadata) =>
                d.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                d.category.toLowerCase().includes(searchTerm.toLowerCase())
            )
            : docs;

        filtered.forEach((doc: DocMetadata) => {
            if (!groups[doc.category]) groups[doc.category] = [];
            groups[doc.category].push(doc);
        });
        return groups;
    }, [docs, searchTerm]);

    // ── TOC Extraction ────────────────────────────────
    const toc = useMemo((): TOCItem[] => {
        const lines = content.replace(/\r/g, '').split('\n');
        const items: TOCItem[] = [];

        lines.forEach(line => {
            const match = line.match(/^(#{1,4})\s+(.+)$/);
            if (match) {
                const level = match[1].length;
                const text = match[2].trim();
                const id = text
                    .toLowerCase()
                    .replace(/\s*\([^)]*\)/g, '')
                    .replace(/[^\w\s-]/g, '')
                    .replace(/\s+/g, '-')
                    .replace(/^-+|-+$/g, '');
                items.push({ id, text, level });
            }
        });
        return items;
    }, [content]);

    const toggleCategory = (category: string) => {
        setExpandedCategories(prev => ({ ...prev, [category]: !prev[category] }));
    };

    return {
        activeTab,
        set_active_tab,
        searchTerm,
        setSearchTerm,
        selectedDoc,
        setSelectedDoc,
        content,
        isLoading,
        groupedDocs,
        expandedCategories,
        toggleCategory,
        toc
    };
}

// Metadata: [useDocs]
