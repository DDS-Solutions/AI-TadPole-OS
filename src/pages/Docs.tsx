/**
 * @docs ARCHITECTURE:UI-Pages
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Docs]` in observability traces.
 */

import ReactMarkdown from 'react-markdown';
import { Search, ChevronRight, ChevronDown, Book, List, Layout } from 'lucide-react';
import clsx from 'clsx';
import { i18n } from '../i18n';
import { useDocs } from '../hooks/useDocs';

/**
 * Docs
 * Unified Documentation Hub for Tadpole OS.
 * Transitioned to a modular, Markdown-driven CMS architecture.
 */
export default function Docs() {
    const {
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
    } = useDocs();

    const generateSlug = (text: string) => {
        return text
            .toLowerCase()
            .replace(/\s*\([^)]*\)/g, '')
            .replace(/[^\w\s-]/g, '')
            .replace(/\s+/g, '-')
            .replace(/^-+|-+$/g, '');
    };

    const getRawText = (node: unknown): string => {
        if (!node) return '';
        if (typeof node === 'string') return node;
        if (Array.isArray(node)) return node.map(getRawText).join('');
        if (typeof node === 'object' && node !== null) {
            const obj = node as Record<string, unknown>;
            if (obj.props && typeof obj.props === 'object' && obj.props !== null) {
                const props = obj.props as Record<string, unknown>;
                if (props.children) return getRawText(props.children);
            }
        }
        return '';
    };

    const scrollToSection = (id: string) => {
        const element = document.getElementById(id);
        const container = document.querySelector('.docs-content-area');
        if (element && container) {
            const top = (element as HTMLElement).offsetTop - 20;
            container.scrollTo({ top, behavior: 'smooth' });
        }
    };

    return (
        <div className="max-w-7xl mx-auto flex flex-col h-full font-sans antialiased text-zinc-300 p-6 lg:p-8">
            {/* ── Premium Tab Navigation ───────────────────────── */}
            <header className="flex items-center justify-between mb-8 bg-zinc-900/50 backdrop-blur-xl border border-zinc-800/50 p-1 rounded-2xl shadow-2xl w-fit mx-auto lg:mx-0">
                <button
                    onClick={() => set_active_tab('knowledge')}
                    className={clsx(
                        "flex items-center gap-2 px-8 py-2.5 rounded-xl text-xs font-bold uppercase tracking-widest transition-all duration-300",
                        activeTab === 'knowledge'
                            ? "bg-zinc-100 text-zinc-950 shadow-[0_0_30px_rgba(255,255,255,0.1)] scale-105"
                            : "text-zinc-500 hover:text-zinc-300"
                    )}
                >
                    <Book size={14} />
                    {i18n.t('docs.tab_knowledge')}
                </button>
                <button
                    onClick={() => set_active_tab('manual')}
                    className={clsx(
                        "flex items-center gap-2 px-8 py-2.5 rounded-xl text-xs font-bold uppercase tracking-widest transition-all duration-300",
                        activeTab === 'manual'
                            ? "bg-zinc-100 text-zinc-950 shadow-[0_0_30px_rgba(255,255,255,0.1)] scale-105"
                            : "text-zinc-500 hover:text-zinc-300"
                    )}
                >
                    <Layout size={14} />
                    {i18n.t('docs.tab_manual')}
                </button>
            </header>

            <div className="flex gap-8 h-full min-h-0">
                {/* ── Adaptive Sidebar ────────────────────────────── */}
                <aside className="w-72 hidden lg:flex flex-col h-full overflow-hidden animate-in fade-in slide-in-from-left-4 duration-700">
                    {activeTab === 'knowledge' ? (
                        <>
                            <div className="relative mb-6">
                                <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-zinc-600" size={16} />
                                <input
                                    type="text"
                                    placeholder={i18n.t('docs.search_placeholder')}
                                    value={searchTerm}
                                    onChange={(e) => setSearchTerm(e.target.value)}
                                    className="w-full bg-zinc-900/80 border border-zinc-800 rounded-2xl pl-12 pr-4 py-3 text-sm text-zinc-200 focus:outline-none focus:ring-2 focus:ring-zinc-700/50 transition-all placeholder:text-zinc-700 shadow-inner"
                                />
                            </div>

                            <nav className="space-y-6 overflow-y-auto pr-2 custom-scrollbar flex-1">
                                {Object.entries(groupedDocs).map(([category, pages]) => (
                                    <div key={category} className="space-y-2">
                                        <button
                                            onClick={() => toggleCategory(category)}
                                            className="w-full flex items-center justify-between text-[10px] font-black text-zinc-600 uppercase tracking-[0.2em] px-2 hover:text-zinc-400 transition-colors"
                                        >
                                            {category}
                                            {expandedCategories[category] ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                                        </button>

                                        {expandedCategories[category] && (
                                            <div className="space-y-1 ml-2 border-l border-zinc-800/50 pl-2">
                                                {pages.map(doc => (
                                                    <button
                                                        key={doc.name}
                                                        onClick={() => setSelectedDoc(doc)}
                                                        className={clsx(
                                                            "w-full text-left px-4 py-2.5 rounded-xl text-xs transition-all flex items-center gap-3 group",
                                                            selectedDoc?.name === doc.name
                                                                ? "bg-zinc-800/50 text-zinc-100 font-bold border border-zinc-700/30"
                                                                : "text-zinc-500 hover:bg-zinc-800/20 hover:text-zinc-300"
                                                        )}
                                                    >
                                                        <div className={clsx(
                                                            "w-1.5 h-1.5 rounded-full transition-all duration-500",
                                                            selectedDoc?.name === doc.name ? "bg-green-500 shadow-[0_0_10px_rgba(34,197,94,0.5)]" : "bg-zinc-800 group-hover:bg-zinc-700"
                                                        )} />
                                                        {doc.title}
                                                    </button>
                                                ))}
                                            </div>
                                        )}
                                    </div>
                                ))}
                            </nav>
                        </>
                    ) : (
                        <div className="flex flex-col h-full animate-in fade-in slide-in-from-left-4 duration-700">
                            <div className="flex items-center gap-3 text-[10px] font-black text-zinc-600 uppercase tracking-[0.2em] mb-6 px-4">
                                <List size={14} />
                                {i18n.t('docs.manual_sections')}
                            </div>
                            <nav className="space-y-1 overflow-y-auto custom-scrollbar pr-2 flex-1 border-l border-zinc-800/50 ml-4">
                                {toc.map((item) => (
                                    <button
                                        key={item.id}
                                        onClick={() => scrollToSection(item.id)}
                                        className={clsx(
                                            "w-full text-left px-4 py-2 text-xs transition-all hover:text-zinc-100",
                                            item.level === 1 ? "font-bold text-zinc-400 mt-4 first:mt-0" : "text-zinc-600 pl-8 border-l border-transparent hover:border-zinc-700",
                                            item.level === 3 && "pl-12 opacity-80",
                                            item.level === 4 && "pl-16 opacity-60 text-[10px]"
                                        )}
                                    >
                                        {item.text}
                                    </button>
                                ))}
                            </nav>
                        </div>
                    )}
                </aside>

                {/* ── Main Content Area ───────────────────────────── */}
                <main className="flex-1 bg-zinc-900/20 px-8 lg:px-16 py-12 rounded-[2.5rem] border border-zinc-800/50 shadow-2xl min-h-0 overflow-y-auto custom-scrollbar relative animate-in fade-in slide-in-from-bottom-4 duration-700 docs-content-area">
                    {isLoading ? (
                        <div className="flex flex-col items-center justify-center h-full gap-6 text-zinc-700">
                            <div className="w-12 h-12 border-2 border-zinc-800 border-t-zinc-400 rounded-full animate-spin" />
                            <span className="text-[10px] font-black uppercase tracking-[0.4em] animate-pulse">{i18n.t('docs.loading')}</span>
                        </div>
                    ) : (
                        <article className="prose prose-invert prose-zinc max-w-none 
                            prose-headings:text-zinc-100 prose-headings:font-black prose-headings:tracking-tighter
                            prose-h1:text-5xl prose-h1:mb-12 prose-h1:pb-6 prose-h1:border-b prose-h1:border-zinc-800/80
                            prose-h2:text-2xl prose-h2:mt-16 prose-h2:mb-6
                            prose-p:text-zinc-400 prose-p:leading-loose prose-p:text-lg
                            prose-a:text-emerald-400 prose-a:font-bold prose-a:no-underline hover:prose-a:text-blue-400 prose-a:transition-all
                            prose-code:text-emerald-300 prose-code:bg-emerald-500/10 prose-code:px-2 prose-code:py-1 prose-code:rounded-lg prose-code:font-mono prose-code:text-[0.85em] prose-code:before:content-none prose-code:after:content-none
                            prose-pre:bg-zinc-950/80 prose-pre:border prose-pre:border-zinc-800 prose-pre:rounded-3xl prose-pre:shadow-2xl prose-pre:p-8
                            prose-blockquote:border-l-4 prose-blockquote:border-l-emerald-500 prose-blockquote:bg-emerald-500/5 prose-blockquote:py-4 prose-blockquote:px-8 prose-blockquote:rounded-2xl prose-blockquote:italic
                        ">
                            <ReactMarkdown
                                skipHtml
                                components={{
                                    h1: ({ node, ...props }) => { void node; return <h1 id={generateSlug(getRawText(props.children))} {...props} />; },
                                    h2: ({ node, ...props }) => { void node; return <h2 id={generateSlug(getRawText(props.children))} {...props} />; },
                                    h3: ({ node, ...props }) => { void node; return <h3 id={generateSlug(getRawText(props.children))} {...props} />; },
                                    h4: ({ node, ...props }) => { void node; return <h4 id={generateSlug(getRawText(props.children))} {...props} />; },
                                }}
                            >
                                {content}
                            </ReactMarkdown>

                            <footer className="mt-24 pt-12 border-t border-zinc-800/80 flex flex-wrap items-center justify-between gap-8 text-zinc-500 not-prose">
                                <div className="flex items-center gap-4">
                                    <div className="w-12 h-12 rounded-2xl bg-zinc-800 border border-zinc-700/50 flex items-center justify-center text-emerald-400 font-black text-xs shadow-lg">
                                        A9
                                    </div>
                                    <div className="flex flex-col">
                                        <span className="text-[10px] uppercase tracking-[0.2em] font-black text-zinc-600">Archivist</span>
                                        <span className="text-sm font-bold text-zinc-400">Agent of Nine</span>
                                    </div>
                                </div>
                                <div className="flex flex-col md:items-end gap-1">
                                    <span className="text-[10px] uppercase tracking-[0.2em] font-black text-zinc-600">Classification</span>
                                    <span className="text-xs font-mono px-3 py-1 bg-zinc-800/50 rounded-full border border-zinc-700/30 text-emerald-500/80">LEVEL-05 (SOVEREIGN)</span>
                                </div>
                                <div className="flex flex-col md:items-end gap-1">
                                    <span className="text-[10px] uppercase tracking-[0.2em] font-black text-zinc-600">Data Integrity</span>
                                    <time dateTime={new Date().toISOString()} className="text-xs font-mono">
                                        {new Date().toLocaleDateString(undefined, { year: 'numeric', month: 'long', day: 'numeric' })}
                                    </time>
                                </div>
                            </footer>
                        </article>
                    )}
                </main>
            </div>
        </div>
    );
}

// Metadata: [Docs]
