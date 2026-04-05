/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Centralized documentation and SOP (Standard Operating Procedure) hub. 
 * Orchestrates the rendering of technical guides and operational protocols for swarm operators.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Broken deep-links to specific MD files, or search index corruption causing query misses.
 * - **Telemetry Link**: Search for `[Docs_View]` or `load_artifact` in service logs.
 */

/**
 * @page Docs
 * Unified Documentation Hub for Tadpole OS.
 * Features a high-fidelity tabular layout switching between the 
 * categorized Knowledge Base and the comprehensive Operations Manual.
 * Includes dynamic TOC generation and RESTful data synchronization.
 */
import { useState, useMemo, useEffect } from 'react';
import ReactMarkdown from 'react-markdown';
import { Search, ChevronRight, ChevronDown, Book, List, Layout } from 'lucide-react';
import clsx from 'clsx';
import { i18n } from '../i18n';
import { tadpole_os_service } from '../services/tadpoleos_service';

type DocMetadata = {
    category: string;
    name: string;
    title: string;
};

type TOCItem = {
    id: string;
    text: string;
    level: number;
};

export default function Docs() {
    // --- State Management ---
    const [activeTab, set_active_tab] = useState<'knowledge' | 'manual'>('knowledge');
    const [searchTerm, setSearchTerm] = useState('');
    const [docs, setDocs] = useState<DocMetadata[]>([]);
    const [selectedDoc, setSelectedDoc] = useState<DocMetadata | null>(null);
    const [content, setContent] = useState<string>('');
    const [isLoading, setIsLoading] = useState(true);
    const [expandedCategories, setExpandedCategories] = useState<Record<string, boolean>>({});

    // --- Data Fetching ---

    // 1. Initial Load: Fetch document list
    useEffect(() => {
        const fetchDocs = async () => {
            try {
                const docList = await tadpole_os_service.get_knowledge_docs();
                setDocs(docList);

                // Initialize categories as expanded
                const cats = Array.from(new Set(docList.map(d => d.category)));
                const expanded: Record<string, boolean> = {};
                cats.forEach(c => expanded[c] = true);
                setExpandedCategories(expanded);

                // Auto-select first doc if none selected
                if (docList.length > 0) {
                    setSelectedDoc(docList.find(d => d.name.includes('architecture')) || docList[0]);
                }
            } catch (error) {
                console.error('Failed to fetch doc list:', error);
            } finally {
                setIsLoading(false);
            }
        };
        fetchDocs();
    }, []);

    // 2. Fetch Content: When selected doc or tab changes
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
            } catch (error: unknown) {
                console.error('Failed to fetch content:', error);
                const message = error instanceof Error ? error.message : i18n.t('docs.error_network_failure');
                setContent(`
${i18n.t('docs.error_title')}
${i18n.t('docs.error_desc')}

**${i18n.t('docs.error_diagnostics')}**
${i18n.t('docs.error_endpoint', { endpoint: activeTab === 'manual' ? '/docs/operations-manual' : `/docs/knowledge/${selectedDoc?.category}/${selectedDoc?.name}` })}
${i18n.t('docs.error_protocol', { message })}

${i18n.t('docs.error_footer')}
                `);
            } finally {
                setIsLoading(false);
                // Reset scroll position on content change
                const container = document.querySelector('.prose')?.parentElement;
                if (container) container.scrollTop = 0;
            }
        };
        fetchContent();
    }, [activeTab, selectedDoc]);

    // --- Business Logic ---

    const generateSlug = (text: string) => {
        return text
            .toLowerCase()
            // Strip parentheses and their content to match manual's internal links
            .replace(/\s*\([^)]*\)/g, '')
            .replace(/[^\w\s-]/g, '')
            .replace(/\s+/g, '-')
            .replace(/^-+|-+$/g, '');
    };

    const getRawText = (node: unknown): string => {
        if (!node) return '';
        if (typeof node === 'string') return node;
        if (Array.isArray(node)) return node.map(getRawText).join('');
        if (typeof node === 'object' && node !== null && 'props' in node) {
            const props = (node as { props: { children?: unknown } }).props;
            if (props && props.children) return getRawText(props.children);
        }
        return '';
    };

    // Grouping & Filtering
    const groupedDocs = useMemo(() => {
        const groups: Record<string, DocMetadata[]> = {};
        const activeDocs = searchTerm
            ? docs.filter(d =>
                d.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                d.category.toLowerCase().includes(searchTerm.toLowerCase())
            )
            : docs;

        activeDocs.forEach(doc => {
            if (!groups[doc.category]) groups[doc.category] = [];
            groups[doc.category].push(doc);
        });
        return groups;
    }, [docs, searchTerm]);

    // TOC Extraction
    const toc = useMemo((): TOCItem[] => {
        const lines = content.replace(/\r/g, '').split('\n');
        const items: TOCItem[] = [];

        lines.forEach(line => {
            const match = line.match(/^(#{1,4})\s+(.+)$/);
            if (match) {
                const level = match[1].length;
                const text = match[2].trim();
                const id = generateSlug(text);
                items.push({ id, text, level });
            }
        });
        return items;
    }, [content]);

    const scrollToSection = (id: string) => {
        const element = document.getElementById(id);
        const container = document.querySelector('.docs-content-area');
        if (element && container) {
            const top = element.offsetTop - 20; // Slight padding
            container.scrollTo({ top, behavior: 'smooth' });
        } else if (element) {
            element.scrollIntoView({ behavior: 'smooth', block: 'start' });
        }
    };

    const toggleCategory = (category: string) => {
        setExpandedCategories(prev => ({ ...prev, [category]: !prev[category] }));
    };

    return (
        <div className="max-w-7xl mx-auto flex flex-col h-full font-sans antialiased text-zinc-300">

            {/* --- Premium Tab Bar --- */}
            <div className="flex items-center justify-between mb-6 bg-zinc-900/50 backdrop-blur-xl border border-zinc-800/50 p-1.5 rounded-2xl shadow-xl w-fit mx-auto lg:mx-0 shrink-0">
                <button
                    onClick={() => set_active_tab('knowledge')}
                    className={clsx(
                        "flex items-center gap-2 px-6 py-2 rounded-xl text-xs font-medium transition-all duration-300",
                        activeTab === 'knowledge'
                            ? "bg-zinc-100 text-zinc-950 shadow-[0_0_20px_rgba(255,255,255,0.1)]"
                            : "text-zinc-500 hover:text-zinc-300"
                    )}
                >
                    <Book size={14} />
                    {i18n.t('docs.tab_knowledge')}
                </button>
                <button
                    onClick={() => set_active_tab('manual')}
                    className={clsx(
                        "flex items-center gap-2 px-6 py-2 rounded-xl text-xs font-medium transition-all duration-300",
                        activeTab === 'manual'
                            ? "bg-zinc-100 text-zinc-950 shadow-[0_0_20px_rgba(255,255,255,0.1)]"
                            : "text-zinc-500 hover:text-zinc-300"
                    )}
                >
                    <Layout size={14} />
                    {i18n.t('docs.tab_manual')}
                </button>
            </div>

            <div className="flex gap-6 h-full min-h-0">
                {/* --- Shared Sidebar Slot --- */}
                <div className="w-64 hidden lg:block sticky top-0 self-start space-y-6 shrink-0 flex flex-col h-full overflow-hidden animate-in fade-in slide-in-from-left-4 duration-500">

                    {/* Knowledge Sidebar Content */}
                    {activeTab === 'knowledge' && (
                        <>
                            <div className="relative shrink-0">
                                <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500" size={14} />
                                <input
                                    type="text"
                                    placeholder={i18n.t('docs.search_placeholder')}
                                    value={searchTerm}
                                    onChange={(e) => setSearchTerm(e.target.value)}
                                    className="w-full bg-zinc-900/80 border border-zinc-800 rounded-xl pl-9 pr-4 py-2.5 text-xs text-zinc-200 focus:outline-none focus:ring-1 focus:ring-zinc-700 transition-all placeholder:text-zinc-600 shadow-inner"
                                    aria-label={i18n.t('docs.aria_search')}
                                />
                            </div>

                            <div className="space-y-4 overflow-y-auto pr-2 custom-scrollbar flex-1">
                                {Object.entries(groupedDocs).map(([category, pages]) => (
                                    <div key={category} className="space-y-1">
                                        <div
                                            role="button"
                                            tabIndex={0}
                                            onClick={() => toggleCategory(category)}
                                            onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleCategory(category)}
                                            aria-expanded={expandedCategories[category]}
                                            className="flex items-center justify-between text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-2 px-2 cursor-pointer hover:text-zinc-200 transition-colors"
                                            aria-label={i18n.t('docs.aria_toggle_category', { category })}
                                        >
                                            {category}
                                            {expandedCategories[category] ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                                        </div>

                                        {expandedCategories[category] && (
                                            <div className="space-y-0.5 ml-1">
                                                {pages.map(doc => (
                                                    <div
                                                        key={doc.name}
                                                        role="button"
                                                        tabIndex={0}
                                                        onClick={() => setSelectedDoc(doc)}
                                                        onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && setSelectedDoc(doc)}
                                                        className={clsx(
                                                            "px-3 py-2 rounded-lg text-xs cursor-pointer transition-all flex items-center gap-3",
                                                            selectedDoc?.name === doc.name
                                                                ? "bg-zinc-800/80 text-zinc-100 font-medium shadow-sm border border-zinc-700/50"
                                                                : "text-zinc-500 hover:bg-zinc-800/30 hover:text-zinc-300"
                                                        )}
                                                        aria-label={i18n.t('docs.aria_select_doc', { title: doc.title })}
                                                        aria-current={selectedDoc?.name === doc.name ? 'page' : undefined}
                                                    >
                                                        <div className={clsx(
                                                            "w-1 h-3 rounded-full transition-all duration-300",
                                                            selectedDoc?.name === doc.name ? "bg-blue-500" : "bg-transparent"
                                                        )} />
                                                        {doc.title}
                                                    </div>
                                                ))}
                                            </div>
                                        )}
                                    </div>
                                ))}
                            </div>
                        </>
                    )}

                    {/* Operations Manual TOC Sidebar */}
                    {activeTab === 'manual' && (
                        <div className="flex flex-col h-full animate-in fade-in slide-in-from-left-4 duration-500">
                            <div className="flex items-center gap-2 text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-4 px-3">
                                <List size={12} />
                                {i18n.t('docs.manual_sections')}
                            </div>
                            <nav className="space-y-1 overflow-y-auto custom-scrollbar pr-2 flex-1">
                                {toc.length > 0 ? (
                                    toc.map((item) => (
                                        <div
                                            role="button"
                                            tabIndex={0}
                                            onClick={() => scrollToSection(item.id)}
                                            onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && scrollToSection(item.id)}
                                            className={clsx(
                                                "px-3 py-1.5 text-xs rounded-lg cursor-pointer transition-all",
                                                "hover:bg-zinc-800 hover:text-zinc-200",
                                                item.level === 1 ? "font-bold text-zinc-400" : "text-zinc-500 ml-3",
                                                item.level === 3 && "ml-4 opacity-80",
                                                item.level === 4 && "ml-6 opacity-60 text-[10px]"
                                            )}
                                            aria-label={i18n.t('docs.aria_scroll_to', { section: item.text })}
                                        >
                                            {item.text}
                                        </div>
                                    ))
                                ) : (
                                    <div className="px-3 text-xs text-zinc-600">{i18n.t('docs.no_sections')}</div>
                                )}
                            </nav>

                        </div>
                    )}
                </div>

                {/* --- Main Content Area --- */}
                <div className="flex-1 bg-zinc-950/30 px-6 lg:px-12 py-10 rounded-2xl border border-zinc-900 shadow-2xl min-h-0 overflow-y-auto custom-scrollbar relative animate-in fade-in slide-in-from-bottom-2 duration-500 docs-content-area">
                    {isLoading ? (
                        <div className="flex flex-col items-center justify-center h-full gap-4 text-zinc-600">
                            <div className="w-8 h-8 border-2 border-zinc-800 border-t-zinc-400 rounded-full animate-spin" />
                            <span className="text-[10px] font-mono uppercase tracking-[0.2em]">{i18n.t('docs.loading')}</span>
                        </div>
                    ) : (
                        <article className="prose prose-invert prose-zinc max-w-none 
                            prose-headings:text-zinc-100 prose-headings:font-bold prose-headings:tracking-tight
                            prose-h1:text-4xl prose-h1:mb-8 prose-h1:pb-4 prose-h1:border-b prose-h1:border-zinc-800
                            prose-p:text-zinc-400 prose-p:leading-relaxed
                            prose-a:text-blue-400 prose-a:no-underline hover:prose-a:text-blue-300 prose-a:transition-colors
                            prose-code:text-emerald-400 prose-code:bg-zinc-900/80 prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:font-mono prose-code:text-[0.9em] prose-code:before:content-none prose-code:after:content-none
                            prose-pre:bg-zinc-900/50 prose-pre:border prose-pre:border-zinc-800 prose-pre:rounded-2xl prose-pre:shadow-2xl
                            prose-blockquote:border-l-blue-500 prose-blockquote:bg-blue-500/5 prose-blockquote:py-1 prose-blockquote:rounded-r-lg
                        ">
                            <ReactMarkdown
                                components={{
                                    h1: ({ ...props }) => {
                                        const { node: _, ...rest } = props as { node: unknown; children: React.ReactNode };
                                        void _;
                                        return <h1 id={generateSlug(getRawText(props.children))} {...rest} />;
                                    },
                                    h2: ({ ...props }) => {
                                        const { node: _, ...rest } = props as { node: unknown; children: React.ReactNode };
                                        void _;
                                        return <h2 id={generateSlug(getRawText(props.children))} {...rest} />;
                                    },
                                    h3: ({ ...props }) => {
                                        const { node: _, ...rest } = props as { node: unknown; children: React.ReactNode };
                                        void _;
                                        return <h3 id={generateSlug(getRawText(props.children))} {...rest} />;
                                    },
                                    h4: ({ ...props }) => {
                                        const { node: _, ...rest } = props as { node: unknown; children: React.ReactNode };
                                        void _;
                                        return <h4 id={generateSlug(getRawText(props.children))} {...rest} />;
                                    },
                                }}
                            >
                                {content}
                            </ReactMarkdown>
                        </article>
                    )}
                </div>
            </div>
        </div>
    );
}

