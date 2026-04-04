import { useState, useEffect } from 'react';
import { Search, Download, ShieldCheck, Filter, Store, Star, Clock, AlertTriangle, ArrowLeft, Code, X } from 'lucide-react';
import { system_api_service } from '../services/system_api_service';
import { i18n } from '../i18n';

interface Template {
    id: string;
    name: string;
    description: string;
    industry: string;
    company_size?: number;
    tags: string[];
    path: string;
    author?: string;
    updatedAt?: string;
    stars?: number;
    installed?: boolean;
}

const REPO_URL = 'https://github.com/DDS-Solutions/Tadpole-OS-Industry-Templates.git';
const REGISTRY_RAW = 'https://raw.githubusercontent.com/DDS-Solutions/Tadpole-OS-Industry-Templates/main/registry.json';

function Template_Store() {
    const [templates, setTemplates] = useState<Template[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const [searchQuery, setSearchQuery] = useState('');
    const [selectedIndustry, setSelectedIndustry] = useState('All');
    const [selectedCompanySize, setSelectedCompanySize] = useState('All');
    const [isInstalling, setIsInstalling] = useState<string | null>(null);
    const [previewTemplate, setPreviewTemplate] = useState<Template | null>(null);
    const [previewConfig, setPreviewConfig] = useState<Record<string, unknown> | null>(null);
    const [isPreviewLoading, setIsPreviewLoading] = useState(false);

    useEffect(() => {
        const fetchRegistry = async () => {
            try {
                setIsLoading(true);
                const res = await fetch(REGISTRY_RAW);
                if (!res.ok) throw new Error('Failed to load Swarm Template Registry');
                const data = await res.json();
                
                const loaded: Template[] = (data.templates || []).map((t: Partial<Template>) => ({
                    ...t,
                    author: t.author || 'SMB Legal Inc.',
                    updatedAt: t.updatedAt || new Date().toISOString().split('T')[0],
                    stars: t.stars || Math.floor(Math.random() * 500) + 50,
                    installed: false 
                }));
                
                setTemplates(loaded);
            } catch (err) {
                setError(err instanceof Error ? err.message : 'Unknown error');
            } finally {
                setIsLoading(false);
            }
        };
        fetchRegistry();
    }, []);

    const INDUSTRIES = ['All', ...Array.from(new Set(templates.map(t => t.industry)))];
    const COMPANY_SIZES = ['All', '25', '50', '100', '150', '200'];

    const filteredTemplates = templates.filter(t => {
        const matchSearch = t.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
                            t.description.toLowerCase().includes(searchQuery.toLowerCase());
        const matchIndustry = selectedIndustry === 'All' || t.industry.toLowerCase() === selectedIndustry.toLowerCase();
        
        // Match size - if template doesn't have a size, it's only shown if 'All' is selected
        // Or if we want to be permissive, we can show it if it doesn't have a size.
        // The user said they want to filter BY size, so let's be strict if a size is selected.
        const matchSize = selectedCompanySize === 'All' || String(t.company_size) === selectedCompanySize;
        
        return matchSearch && matchIndustry && matchSize;
    });

    const handleInstall = async (template: Template) => {
        try {
            setIsInstalling(template.id);
            await system_api_service.install_template(REPO_URL, template.path);

            // Mark as installed in local state
            setTemplates(prev => prev.map(t => t.id === template.id ? { ...t, installed: true } : t));
            
            // Dispatch a global event so the UI knows to refresh the agents panel if needed
            window.dispatchEvent(new Event('app:refresh-agents'));
            
            // Close preview if it was open
            setPreviewTemplate(null);
            setPreviewConfig(null);
        } catch (err: unknown) {
            const msg = err instanceof Error ? err.message : String(err);
            alert(`Error installing template: ${msg}`);
        } finally {
            setIsInstalling(null);
        }
    };

    const handlePreview = async (template: Template) => {
        console.debug('handlePreview invoked for:', template.id);
        try {
            setPreviewTemplate(template);
            setIsPreviewLoading(true);
            setPreviewConfig(null);
            
            const configUrl = `https://raw.githubusercontent.com/DDS-Solutions/Tadpole-OS-Industry-Templates/main/${template.path}/swarm.json`;
            console.debug('Fetching configUrl:', configUrl);
            const res = await fetch(configUrl);
            if (!res.ok) throw new Error('Failed to fetch swarm configuration');
            
            const config = await res.json();
            console.debug('Config loaded successfully');
            setPreviewConfig(config);
        } catch (err) {
            console.error('Preview error:', err);
            setError('Could not load configuration preview.');
        } finally {
            setIsPreviewLoading(false);
        }
    };

    return (
        <div className="h-full flex flex-col pt-6 px-6">
            {/* Header */}
            <div className="flex-none mb-8">
                <div className="flex items-center justify-between mb-4">
                    <div>
                        <h1 className="text-2xl font-bold text-zinc-100 flex items-center gap-3 tracking-tight">
                            <Store className="text-blue-500" size={28} />
                            {i18n.t('template_store.title')}
                        </h1>
                        <p className="text-sm text-zinc-400 mt-1">{i18n.t('template_store.desc')}</p>
                    </div>
                    <div className="flex items-center gap-2 px-3 py-1.5 bg-zinc-900 rounded-full border border-zinc-800 text-xs font-mono text-zinc-400">
                        <ShieldCheck size={14} className="text-emerald-500" />
                        {i18n.t('template_store.shield_active')}
                    </div>
                </div>

                {/* Search & Filters */}
                <div className="flex flex-col md:flex-row gap-4">
                    <div className="relative flex-1">
                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500" size={18} />
                        <input
                            type="text"
                            placeholder={i18n.t('template_store.search_placeholder')}
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            className="w-full bg-zinc-900/50 border border-zinc-800 rounded-lg py-2.5 pl-10 pr-4 text-sm text-zinc-100 focus:outline-none focus:border-blue-500 transition-colors"
                        />
                    </div>
                    <div className="flex flex-col gap-4 flex-shrink-0">
                        <div data-testid="industry-filters" className="flex items-center gap-2 overflow-x-auto pb-2 md:pb-0 custom-scrollbar">
                            <Filter className="text-zinc-500 mr-2 flex-shrink-0" size={16} />
                            {INDUSTRIES.map(ind => (
                                <button
                                    key={ind}
                                    onClick={() => setSelectedIndustry(ind)}
                                    className={`px-4 py-1.5 rounded-full text-xs font-bold whitespace-nowrap transition-colors ${
                                        selectedIndustry === ind 
                                            ? 'bg-blue-600 text-white shadow-lg shadow-blue-500/20' 
                                            : 'bg-zinc-900 text-zinc-400 hover:bg-zinc-800 border border-zinc-800'
                                    }`}
                                >
                                    {ind}
                                </button>
                            ))}
                        </div>
                        <div data-testid="size-filters" className="flex items-center gap-2 overflow-x-auto pb-2 md:pb-0 custom-scrollbar">
                            <span className="text-[10px] uppercase tracking-widest font-bold text-zinc-500 mr-2 flex-shrink-0">{i18n.t('template_store.label_size')}</span>
                            {COMPANY_SIZES.map(size => (
                                <button
                                    key={size}
                                    onClick={() => setSelectedCompanySize(size)}
                                    className={`px-3 py-1 rounded-md text-[10px] font-bold whitespace-nowrap transition-all ${
                                        selectedCompanySize === size 
                                            ? 'bg-emerald-600 text-white shadow-lg shadow-emerald-500/20' 
                                            : 'bg-zinc-900 text-zinc-500 hover:bg-zinc-800 border border-zinc-800'
                                    }`}
                                >
                                    {size}{size !== 'All' ? ` ${i18n.t('template_store.employees')}` : ''}
                                </button>
                            ))}
                        </div>
                    </div>
                </div>
            </div>

            {/* Grid */}
            <div className="flex-1 overflow-y-auto custom-scrollbar pb-8">
                {isLoading ? (
                    <div className="flex flex-col items-center justify-center py-20 text-zinc-500">
                        <div className="w-8 h-8 rounded-full border-2 border-blue-500 border-t-transparent animate-spin mb-4"></div>
                        <p>{i18n.t('template_store.connecting')}</p>
                    </div>
                ) : error ? (
                    <div className="flex flex-col items-center justify-center py-20 text-red-400">
                        <AlertTriangle size={48} className="mb-4 opacity-50" />
                        <p>{error}</p>
                    </div>
                ) : (
                    <>
                        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                            {filteredTemplates.map(template => (
                        <div key={template.id} className="bg-zinc-900 border border-zinc-800 rounded-xl p-5 flex flex-col hover:border-zinc-700 transition-colors group">
                            <div className="flex justify-between items-start mb-3">
                                <div>
                                    <div className="flex gap-2">
                                        <span className="text-[10px] uppercase tracking-wider font-bold text-blue-400 bg-blue-500/10 px-2 py-0.5 rounded">
                                            {template.industry}
                                        </span>
                                        {template.company_size && (
                                            <span className="text-[10px] uppercase tracking-wider font-bold text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded">
                                                {template.company_size} {i18n.t('template_store.seats')}
                                            </span>
                                        )}
                                    </div>
                                    <h3 className="text-lg font-bold text-zinc-100 mt-2">{template.name}</h3>
                                </div>
                                <div className="flex items-center gap-1 text-xs text-zinc-500 font-mono">
                                    <Star size={12} className="text-amber-500" />
                                    {template.stars}
                                </div>
                            </div>
                            
                            <p className="text-sm text-zinc-400 leading-relaxed mb-6 flex-1">
                                {template.description}
                            </p>

                            <div className="mt-auto space-y-4">
                                <div className="flex justify-between items-center text-xs text-zinc-500 font-mono border-t border-zinc-800/50 pt-4">
                                    <span>By {template.author}</span>
                                    <span className="flex items-center gap-1"><Clock size={12}/> {template.updatedAt}</span>
                                </div>
                                
                                <button
                                    disabled={template.installed || isInstalling === template.id}
                                    onClick={() => handlePreview(template)}
                                    className={`w-full py-2 rounded-lg font-bold text-sm flex items-center justify-center gap-2 transition-all ${
                                        template.installed 
                                            ? 'bg-zinc-800 text-zinc-500 cursor-not-allowed' 
                                            : isInstalling === template.id
                                                ? 'bg-blue-600/50 text-white cursor-wait'
                                                : 'bg-zinc-100 text-zinc-900 hover:bg-white'
                                    }`}
                                >
                                    {template.installed ? (
                                        <>{i18n.t('template_store.btn_installed')}</>
                                    ) : isInstalling === template.id ? (
                                        <>{i18n.t('template_store.btn_downloading')}</>
                                    ) : (
                                        <>
                                            <Code size={16} />
                                            {i18n.t('template_store.btn_preview')}
                                        </>
                                    )}
                                </button>
                            </div>
                        </div>
                    ))}
                </div>
                {filteredTemplates.length === 0 && (
                    <div className="flex flex-col items-center justify-center py-20 text-zinc-500">
                        <Search size={48} className="mb-4 opacity-20" />
                        <p>{i18n.t('template_store.empty_title')}</p>
                    </div>
                )}
                    </>
                )}
            </div>

            {/* Preview Modal */}
            {previewTemplate && (
                <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4 animate-in fade-in duration-200">
                    <div className="bg-zinc-950 border border-zinc-800 rounded-2xl w-full max-w-4xl max-h-[90vh] flex flex-col shadow-2xl overflow-hidden relative">
                        <button 
                            onClick={() => { setPreviewTemplate(null); setPreviewConfig(null); }}
                            className="absolute top-4 right-4 text-zinc-500 hover:text-zinc-100 transition-colors p-2 hover:bg-zinc-900 rounded-full z-10"
                        >
                            <X size={20} />
                        </button>

                        {/* Modal Header */}
                        <div className="p-8 border-b border-zinc-800/50">
                            <div className="flex items-center gap-3 mb-4">
                                <span className="text-[10px] uppercase tracking-wider font-bold text-blue-400 bg-blue-500/10 px-2 py-0.5 rounded">
                                    {previewTemplate.industry}
                                </span>
                                {previewTemplate.company_size && (
                                    <span className="text-[10px] uppercase tracking-wider font-bold text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded">
                                        {previewTemplate.company_size} {i18n.t('template_store.seats')}
                                    </span>
                                )}
                            </div>
                            <h2 className="text-3xl font-bold text-white tracking-tight mb-2">{previewTemplate.name}</h2>
                            <p className="text-zinc-400 max-w-2xl leading-relaxed">{previewTemplate.description}</p>
                        </div>

                        {/* Modal Content - Scrollable */}
                        <div className="flex-1 overflow-y-auto p-8 custom-scrollbar">
                            <div className="flex items-center gap-2 mb-4 text-xs font-bold text-zinc-500 uppercase tracking-widest">
                                <Code size={14} className="text-blue-500" />
                                Swarm Configuration (swarm.json)
                            </div>
                            
                            <div className="bg-black/40 border border-zinc-800 rounded-xl overflow-hidden relative">
                                {isPreviewLoading ? (
                                    <div className="py-20 flex flex-col items-center justify-center text-zinc-500">
                                        <div className="w-6 h-6 rounded-full border-2 border-blue-500 border-t-transparent animate-spin mb-3"></div>
                                        <p className="text-xs font-mono">{i18n.t('template_store.modal_fetching')}</p>
                                    </div>
                                ) : previewConfig ? (
                                    <pre className="p-6 text-sm font-mono text-blue-300 overflow-x-auto whitespace-pre-wrap leading-relaxed">
                                        {JSON.stringify(previewConfig, null, 2)}
                                    </pre>
                                ) : (
                                    <div className="py-20 flex flex-col items-center justify-center text-red-400">
                                        <AlertTriangle size={32} className="mb-3 opacity-50" />
                                        <p className="text-xs font-mono">{i18n.t('template_store.modal_fail_resolve')}</p>
                                    </div>
                                )}
                            </div>

                            <div className="mt-8 grid grid-cols-1 md:grid-cols-2 gap-6">
                                <div className="p-4 bg-zinc-900/50 border border-zinc-800/50 rounded-xl">
                                    <h4 className="text-xs font-bold text-zinc-100 mb-2 flex items-center gap-2">
                                        <ShieldCheck size={14} className="text-emerald-500" />
                                        {i18n.t('template_store.security_verified_title')}
                                    </h4>
                                    <p className="text-[11px] text-zinc-500 leading-normal">
                                        {i18n.t('template_store.security_verified_desc')}
                                    </p>
                                </div>
                                <div className="p-4 bg-zinc-900/50 border border-zinc-800/50 rounded-xl">
                                    <h4 className="text-xs font-bold text-zinc-100 mb-2 flex items-center gap-2">
                                        <Download size={14} className="text-blue-500" />
                                        {i18n.t('template_store.hot_loading_title')}
                                    </h4>
                                    <p className="text-[11px] text-zinc-500 leading-normal">
                                        {i18n.t('template_store.hot_loading_desc')}
                                    </p>
                                </div>
                            </div>
                        </div>

                        {/* Modal Footer */}
                        <div className="p-6 bg-zinc-900/30 border-t border-zinc-800/50 flex items-center justify-between gap-4">
                            <button
                                onClick={() => { setPreviewTemplate(null); setPreviewConfig(null); }}
                                className="px-6 py-2.5 rounded-lg font-bold text-sm text-zinc-400 hover:text-white hover:bg-zinc-900 transition-all flex items-center gap-2"
                            >
                                <ArrowLeft size={16} />
                                {i18n.t('template_store.btn_back')}
                            </button>
                            
                            <button
                                disabled={isInstalling === previewTemplate.id}
                                onClick={() => handleInstall(previewTemplate)}
                                className={`px-8 py-2.5 rounded-lg font-bold text-sm flex items-center gap-2 shadow-lg transition-all ${
                                    isInstalling === previewTemplate.id
                                        ? 'bg-emerald-600/50 text-white cursor-wait opacity-50'
                                        : 'bg-emerald-600 text-white hover:bg-emerald-500 shadow-emerald-500/20'
                                }`}
                            >
                                {isInstalling === previewTemplate.id ? (
                                    <>
                                        <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                                        {i18n.t('template_store.btn_deploying')}
                                    </>
                                ) : (
                                    <>
                                        <Download size={18} />
                                        {i18n.t('template_store.btn_install')}
                                    </>
                                )}
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}

export default Template_Store;
