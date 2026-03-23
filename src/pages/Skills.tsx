import { useState, useEffect } from 'react';
import { Settings, Plus, Trash2, Edit2, Code, FileText, AlertTriangle, Activity, Terminal, Shield, FlaskConical, Play, Gauge, X, Search, Users, Check } from 'lucide-react';
import { useSkillStore, type SkillDefinition, type WorkflowDefinition, type McpToolHubDefinition, type HookDefinition } from '../stores/skillStore';
import { useAgentStore } from '../stores/agentStore';
import type { Agent } from '../types';
import { TwEmptyState, Tooltip } from '../components/ui';
import { i18n } from '../i18n';

import { TadpoleOSService } from '../services/tadpoleosService';
import { AgentApiService } from '../services/AgentApiService';
import { ImportPreviewModal } from '../components/ImportPreviewModal';
import { Upload } from 'lucide-react';

export default function Skills() {
    const { scripts, workflows, mcpTools, manifests, hooks, isLoading, error, fetchSkills, fetchMcpTools, saveSkillScript, deleteSkillScript, saveWorkflow, deleteWorkflow, saveHook, deleteHook } = useSkillStore();
    const { agents, updateAgent } = useAgentStore();
    const [activeTab, setActiveTab] = useState<'skills' | 'workflows' | 'hooks' | 'mcp'>('skills');
    const [activeCategory, setActiveCategory] = useState<'user' | 'ai'>('user');
    const [searchQuery, setSearchQuery] = useState('');

    // Assignment Modal State
    const [isAssignModalOpen, setIsAssignModalOpen] = useState(false);
    const [assignTarget, setAssignTarget] = useState<{ type: 'skill' | 'workflow' | 'mcp', name: string } | null>(null);

    // Skill Modal State
    const [isSkillModalOpen, setIsSkillModalOpen] = useState(false);
    const [editingSkill, setEditingSkill] = useState<Partial<SkillDefinition>>({});
    const [skillSaveError, setSkillSaveError] = useState<string | null>(null);
    const [schemaError, setSchemaError] = useState<string | null>(null);

    // Workflow Modal State
    const [isWfModalOpen, setIsWfModalOpen] = useState(false);
    const [editingWf, setEditingWf] = useState<Partial<WorkflowDefinition>>({});
    const [wfSaveError, setWfSaveError] = useState<string | null>(null);

    // Hook Modal State
    const [isHookModalOpen, setIsHookModalOpen] = useState(false);
    const [editingHook, setEditingHook] = useState<Partial<HookDefinition>>({});
    const [hookSaveError, setHookSaveError] = useState<string | null>(null);
    const [hookForm, setHookForm] = useState<HookDefinition>({
        name: '',
        description: '',
        hook_type: 'pre_validation',
        content: '',
        active: true,
        category: 'user'
    });

    // Shared State
    const [isSaving, setIsSaving] = useState(false);

    // Import State
    const [isImportModalOpen, setIsImportModalOpen] = useState(false);
    const [importType, setImportType] = useState('');
    const [importData, setImportData] = useState<SkillDefinition | WorkflowDefinition | HookDefinition | null>(null);
    const [importPreview, setImportPreview] = useState('');

    // Tool Lab State
    const [labTool, setLabTool] = useState<McpToolHubDefinition | null>(null);
    const [isLabOpen, setIsLabOpen] = useState(false);
    const [labInput, setLabInput] = useState<string>('{}');
    const [labResult, setLabResult] = useState<Record<string, unknown> | null>(null);
    const [isLabRunning, setIsLabRunning] = useState(false);
    const [labSchemaError, setLabSchemaError] = useState<string | null>(null);

    useEffect(() => {
        fetchSkills();
        fetchMcpTools();
    }, [fetchSkills, fetchMcpTools]);

    const handleRunTool = async () => {
        if (!labTool) return;
        setLabResult(null);
        setLabSchemaError(null);
        setIsLabRunning(true);
        try {
            const parsedArgs = JSON.parse(labInput);
            const data = await TadpoleOSService.executeMcpTool(labTool.name, parsedArgs);
            setLabResult(data as Record<string, unknown> | null);
        } catch (e: unknown) {
            setLabSchemaError(e instanceof Error ? e.message : 'Execution failed');
        } finally {
            setIsLabRunning(false);
        }
    };

    const generateSampleInputs = (schema?: { properties?: Record<string, { type: string }> }) => {
        const inputs: Record<string, unknown> = {};
        if (schema && schema.properties) {
            Object.keys(schema.properties).forEach(key => {
                const prop = schema.properties?.[key];
                if (!prop) return;
                if (prop.type === 'string') inputs[key] = "";
                else if (prop.type === 'number' || prop.type === 'integer') inputs[key] = 0;
                else if (prop.type === 'boolean') inputs[key] = false;
                else if (prop.type === 'object') inputs[key] = {};
                else if (prop.type === 'array') inputs[key] = [];
            });
        }
        return inputs;
    };

    const handleSaveSkill = async () => {
        if (!editingSkill.name || !editingSkill.execution_command) return;
        setSkillSaveError(null);
        setIsSaving(true);
        try {
            await saveSkillScript({ ...editingSkill, category: activeCategory } as SkillDefinition);
            setIsSkillModalOpen(false);
        } catch (e: unknown) {
            setSkillSaveError(e instanceof Error ? e.message : 'Unknown error');
        } finally {
            setIsSaving(false);
        }
    };

    const handleToggleAssignment = async (agentId: string) => {
        if (!assignTarget) return;
        const agent = agents.find(a => a.id === agentId);
        if (!agent) return;

        const { type, name } = assignTarget;
        const updates: Partial<Agent> = {};

        if (type === 'skill') {
            const currentSkills = agent.skills || [];
            updates.skills = currentSkills.includes(name)
                ? currentSkills.filter(s => s !== name)
                : [...currentSkills, name];
        } else if (type === 'workflow') {
            const currentWorkflows = agent.workflows || [];
            updates.workflows = currentWorkflows.includes(name)
                ? currentWorkflows.filter(w => w !== name)
                : [...currentWorkflows, name];
        } else if (type === 'mcp') {
            const currentMcp = agent.mcpTools || [];
            updates.mcpTools = currentMcp.includes(name)
                ? currentMcp.filter(t => t !== name)
                : [...currentMcp, name];
        }

        await updateAgent(agentId, updates);
    };

    const handleSaveWf = async () => {
        if (!editingWf.name || !editingWf.content) return;
        setWfSaveError(null);
        setIsSaving(true);
        try {
            await saveWorkflow({ ...editingWf, category: activeCategory } as WorkflowDefinition);
            setIsWfModalOpen(false);
        } catch (e: unknown) {
            setWfSaveError(e instanceof Error ? e.message : 'Unknown error');
        } finally {
            setIsSaving(false);
        }
    };

    const handleSaveHook = async () => {
        setIsSaving(true);
        setHookSaveError(null);
        try {
            await saveHook(hookForm);
            setIsHookModalOpen(false);
        } catch (e: unknown) {
            setHookSaveError(e instanceof Error ? e.message : 'Failed to save hook');
        } finally {
            setIsSaving(false);
        }
    };

    const handleImportClick = () => {
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = '.md,.py,.js,.ts';
        input.onchange = async (e) => {
            const file = (e.target as HTMLInputElement).files?.[0];
            if (!file) return;

            setIsSaving(true);
            try {
                const result = await AgentApiService.importCapability(file);
                setImportType(result.type);
                setImportData(result.data);
                setImportPreview(result.preview);
                setIsImportModalOpen(true);
            } catch (err: unknown) {
                const message = err instanceof Error ? err.message : String(err);
                alert(`Import failed: ${message}`);
            } finally {
                setIsSaving(false);
            }
        };
        input.click();
    };

    const handleConfirmImport = async (finalData: SkillDefinition | WorkflowDefinition | HookDefinition, category: string) => {
        setIsSaving(true);
        try {
            await AgentApiService.registerCapability(importType, finalData, category);
            setIsImportModalOpen(false);
            await fetchSkills(); 
            await fetchMcpTools();
        } catch (err: unknown) {
            const message = err instanceof Error ? err.message : String(err);
            alert(`Registration failed: ${message}`);
        } finally {
            setIsSaving(false);
        }
    };

    const handleDeleteHook = async (name: string) => {
        if (window.confirm(`Delete Lifecycle Hook ${name}?`)) {
            await deleteHook(name);
        }
    };

    return (
        <div className="flex flex-col h-full gap-6">
            {/* Standardized Module Header */}
            <div className="flex items-center justify-between border-b border-zinc-900 pb-2 px-1 shrink-0">
                <div>
                    <h2 className="text-xl font-bold flex items-center gap-2 text-zinc-100">
                        <Tooltip content={i18n.t('skills.tooltip_main')} position="right">
                            <Settings className="text-blue-500 cursor-help" />
                        </Tooltip>
                        {i18n.t('skills.title')}
                    </h2>
                    <div className="mt-1">
                    </div>
                </div>

                {/* Category Switcher */}
                <div className="flex items-center gap-4">
                    <div className="flex items-center p-1 bg-zinc-900 rounded-lg border border-zinc-800 self-center">
                        <button
                            onClick={() => setActiveCategory('user')}
                            className={`px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-md transition-all flex items-center gap-2 ${activeCategory === 'user'
                                ? 'bg-blue-600 text-white shadow-lg'
                                : 'text-zinc-500 hover:text-zinc-300'
                                }`}
                        >
                            User Registry
                            <span className={`px-1 rounded ${activeCategory === 'user' ? 'bg-black/20 text-white' : 'bg-zinc-800 text-zinc-600'}`}>
                                {scripts.filter(s => s.category === 'user').length + workflows.filter(w => w.category === 'user').length}
                            </span>
                        </button>
                        <button
                            onClick={() => setActiveCategory('ai')}
                            className={`px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-md transition-all flex items-center gap-2 ${activeCategory === 'ai'
                                ? 'bg-indigo-600 text-white shadow-lg'
                                : 'text-zinc-500 hover:text-zinc-300'
                                }`}
                        >
                            AI Services
                            <span className={`px-1 rounded ${activeCategory === 'ai' ? 'bg-black/20 text-white' : 'bg-zinc-800 text-zinc-600'}`}>
                                {manifests.filter(m => m.category === 'ai').length + workflows.filter(w => w.category === 'ai').length}
                            </span>
                        </button>
                    </div>

                    <div className="flex items-center gap-2">
                        <button
                            onClick={handleImportClick}
                            disabled={isSaving}
                            className="bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-300 px-3 py-1.5 rounded-lg text-xs font-bold flex items-center gap-2 transition-all hover:border-indigo-500/50"
                        >
                            <Upload className="w-3.5 h-3.5" /> Import .md
                        </button>
                    </div>

                    <div className="relative">
                        <input
                            type="text"
                            placeholder={i18n.t('agent_manager.placeholder_search')}
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            className="bg-zinc-900 border border-zinc-800 rounded-lg pl-8 pr-3 py-1.5 text-xs text-zinc-200 focus:outline-none focus:border-blue-500/50 w-48 transition-all"
                        />
                        <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 text-zinc-600" size={12} />
                    </div>
                </div>
            </div>

            {error && (
                <div className="bg-red-900/10 border border-red-500/20 p-4 rounded-xl text-red-400 mb-2 flex items-start gap-3">
                    <AlertTriangle className="w-5 h-5 flex-shrink-0 mt-0.5 text-red-500" />
                    <p>{error}</p>
                </div>
            )}

            {/* Tabs */}
            <div className="flex border-b border-zinc-800 shrink-0">
                <Tooltip content={i18n.t('skills.tooltip_skills')} position="bottom">
                    <button
                        onClick={() => setActiveTab('skills')}
                        className={`flex items-center gap-2 px-6 py-3 font-medium transition-colors border-b-2 ${activeTab === 'skills' ? 'border-blue-500 text-blue-400' : 'border-transparent text-zinc-500 hover:text-zinc-300'
                            }`}
                    >
                        <Code className="w-4 h-4" /> {i18n.t('skills.tab_skills')}
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('skills.tooltip_workflows')} position="bottom">
                    <button
                        onClick={() => setActiveTab('workflows')}
                        className={`flex items-center gap-2 px-6 py-3 font-medium transition-colors border-b-2 ${activeTab === 'workflows' ? 'border-blue-500 text-blue-400' : 'border-transparent text-zinc-500 hover:text-zinc-300'
                            }`}
                    >
                        <FileText className="w-4 h-4" /> {i18n.t('skills.tab_workflows')}
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('skills.tooltip_hooks')} position="bottom">
                    <button
                        onClick={() => setActiveTab('hooks')}
                        className={`flex items-center gap-2 px-6 py-3 font-medium transition-colors border-b-2 ${activeTab === 'hooks' ? 'border-blue-500 text-blue-400' : 'border-transparent text-zinc-500 hover:text-zinc-300'
                            }`}
                    >
                        <Shield className="w-4 h-4" /> {i18n.t('skills.tab_hooks')}
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('skills.tooltip_mcp')} position="bottom">
                    <button
                        onClick={() => setActiveTab('mcp')}
                        className={`flex items-center gap-2 px-6 py-3 font-medium transition-colors border-b-2 ${activeTab === 'mcp' ? 'border-cyan-500 text-cyan-400' : 'border-transparent text-zinc-500 hover:text-zinc-300'
                            }`}
                    >
                        <Terminal className="w-4 h-4" /> {i18n.t('skills.tab_mcp')}
                    </button>
                </Tooltip>
            </div>

            {/* Content Area */}
            <div className="flex-1 overflow-y-auto min-h-0 custom-scrollbar pr-2 pb-6">
                {isLoading ? (
                    <div className="flex justify-center p-12"><div className="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"></div></div>
                ) : activeTab === 'skills' ? (
                    <div className="space-y-6">
                        <div className="flex justify-between items-center mb-6 sticky top-0 bg-zinc-950/80 backdrop-blur-md pt-2 pb-3 border-b border-zinc-800/50 z-20">
                            <h3 className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                                <Activity size={12} className="text-blue-500" /> {i18n.t('skills.header_execution')}
                            </h3>
                            <Tooltip content={i18n.t('skills.tooltip_new_skill')} position="bottom">
                                <button
                                    onClick={() => {
                                        setEditingSkill({ schema: { type: "object", properties: {} } });
                                        setSkillSaveError(null);
                                        setSchemaError(null);
                                        setIsSkillModalOpen(true);
                                    }}
                                    className="bg-blue-600 hover:bg-blue-500 text-white px-3 py-1.5 rounded-lg text-xs font-bold flex items-center gap-2 transition-colors shadow-[0_0_15px_rgba(37,99,235,0.3)]"
                                >
                                    <Plus className="w-3.5 h-3.5" /> {i18n.t('skills.btn_new_skill')}
                                </button>
                            </Tooltip>
                        </div>

                        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 relative z-10">
                            {scripts.filter(s => s.category === activeCategory && (s.name.toLowerCase().includes(searchQuery.toLowerCase()) || s.description?.toLowerCase().includes(searchQuery.toLowerCase()))).map(skill => (
                                <div key={skill.name} className="bg-zinc-950 border border-zinc-800 p-5 rounded-xl transition-all duration-300 hover:border-emerald-500/30 hover:shadow-[0_0_15px_rgba(16,185,129,0.15)] group relative overflow-hidden shadow-sm">
                                    <div className="neural-grid opacity-[0.03]" />
                                    <div className="absolute top-4 right-4 flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity z-20">
                                        <Tooltip content={i18n.t('skills.tooltip_edit_skill')} position="top">
                                            <button onClick={() => { setEditingSkill(skill); setSkillSaveError(null); setSchemaError(null); setIsSkillModalOpen(true); }} className="text-zinc-500 hover:text-blue-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded"><Edit2 className="w-3.5 h-3.5" /></button>
                                        </Tooltip>
                                        <Tooltip content={i18n.t('agent_manager.tooltip_assign')} position="top">
                                            <button onClick={() => { setAssignTarget({ type: 'skill', name: skill.name }); setIsAssignModalOpen(true); }} className="text-zinc-500 hover:text-emerald-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded transition-colors"><Users className="w-3.5 h-3.5" /></button>
                                        </Tooltip>
                                        <Tooltip content={i18n.t('skills.tooltip_delete_skill')} position="top">
                                            <button onClick={() => deleteSkillScript(skill.name)} className="text-zinc-500 hover:text-red-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded"><Trash2 className="w-3.5 h-3.5" /></button>
                                        </Tooltip>
                                    </div>
                                    <div className="relative z-10">
                                        <div className="flex items-center gap-3 mb-2 pr-16 text-zinc-300 font-bold tracking-wide">
                                            <div className="w-2 h-2 rounded-full bg-emerald-500/30 group-hover:bg-emerald-400 group-hover:shadow-[0_0_8px_rgba(16,185,129,0.5)] transition-all shrink-0 mt-0.5"></div>
                                            <h3 className="font-mono text-sm">{skill.name}</h3>
                                        </div>
                                        <p className="text-zinc-500 text-xs line-clamp-2 mb-4 h-8 leading-relaxed font-mono">{skill.description}</p>
                                        <div className="bg-black/40 border border-zinc-800/50 p-2.5 rounded font-mono text-[10px] text-zinc-300 flex items-center gap-2 overflow-x-auto">
                                            <Terminal className="w-3 h-3 flex-shrink-0 text-zinc-500" />
                                            <span className="whitespace-nowrap">{skill.execution_command}</span>
                                        </div>
                                    </div>
                                </div>
                            ))}
                             {scripts.length === 0 && <TwEmptyState title={i18n.t('skills.empty_skills_title')} description={i18n.t('skills.empty_skills_desc')} />}
                        </div>
                    </div>
                ) : activeTab === 'workflows' ? (
                    <div className="space-y-6">
                        <div className="flex justify-between items-center mb-6 sticky top-0 bg-zinc-950/80 backdrop-blur-md pt-2 pb-3 border-b border-zinc-800/50 z-20">
                            <h3 className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                                <Activity size={12} className="text-blue-500" /> {i18n.t('skills.header_guiding')}
                            </h3>
                            <Tooltip content={i18n.t('skills.tooltip_new_workflow')} position="bottom">
                                <button
                                    onClick={() => {
                                        setEditingWf({});
                                        setWfSaveError(null);
                                        setIsWfModalOpen(true);
                                    }}
                                    className="bg-blue-600 hover:bg-blue-500 text-white px-3 py-1.5 rounded-lg text-xs font-bold flex items-center gap-2 transition-colors shadow-[0_0_15px_rgba(37,99,235,0.3)]"
                                >
                                    <Plus className="w-3.5 h-3.5" /> {i18n.t('skills.btn_new_workflow')}
                                </button>
                            </Tooltip>
                        </div>

                        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 relative z-10">
                            {workflows.filter(w => w.category === activeCategory && (w.name.toLowerCase().includes(searchQuery.toLowerCase()) || w.content.toLowerCase().includes(searchQuery.toLowerCase()))).map(wf => (
                                <div key={wf.name} className="bg-zinc-950 border border-zinc-800 p-5 rounded-xl transition-all duration-300 hover:border-amber-500/30 hover:shadow-[0_0_15px_rgba(245,158,11,0.15)] group relative overflow-hidden shadow-sm">
                                    <div className="neural-grid opacity-[0.03]" />
                                    <div className="absolute top-4 right-4 flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity z-20">
                                        <Tooltip content={i18n.t('skills.tooltip_edit_workflow')} position="top">
                                            <button onClick={() => { setEditingWf(wf); setWfSaveError(null); setIsWfModalOpen(true); }} className="text-zinc-500 hover:text-blue-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded"><Edit2 className="w-3.5 h-3.5" /></button>
                                        </Tooltip>
                                        <Tooltip content={i18n.t('agent_manager.tooltip_assign')} position="top">
                                            <button onClick={() => { setAssignTarget({ type: 'workflow', name: wf.name }); setIsAssignModalOpen(true); }} className="text-zinc-500 hover:text-emerald-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded transition-colors"><Users className="w-3.5 h-3.5" /></button>
                                        </Tooltip>
                                        <Tooltip content={i18n.t('skills.tooltip_delete_workflow')} position="top">
                                            <button onClick={() => deleteWorkflow(wf.name)} className="text-zinc-500 hover:text-red-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded"><Trash2 className="w-3.5 h-3.5" /></button>
                                        </Tooltip>
                                    </div>
                                    <div className="relative z-10 flex flex-col h-full">
                                        <div className="flex items-center gap-3 mb-3 pr-16 text-zinc-300 font-bold tracking-wide">
                                            <div className="w-2 h-2 rounded-full bg-amber-500/30 group-hover:bg-amber-400 group-hover:shadow-[0_0_8px_rgba(245,158,11,0.5)] transition-all shrink-0 mt-0.5"></div>
                                            <h3 className="font-mono text-sm">{wf.name}</h3>
                                        </div>
                                        <div className="bg-black/40 border border-zinc-800/50 p-3 rounded text-[11px] text-zinc-400 font-mono whitespace-pre-wrap max-h-64 overflow-y-auto custom-scrollbar flex-1">
                                            {wf.content}
                                        </div>
                                    </div>
                                </div>
                            ))}
                            {workflows.length === 0 && <TwEmptyState title={i18n.t('skills.empty_workflows_title')} description={i18n.t('skills.empty_workflows_desc')} />}
                        </div>
                    </div>
                ) : activeTab === 'hooks' ? (
                    <div className="space-y-6">
                        <div className="flex justify-between items-center mb-6 sticky top-0 bg-zinc-950/80 backdrop-blur-md pt-2 pb-3 border-b border-zinc-800/50 z-20">
                            <h3 className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                                <Shield size={12} className="text-emerald-500" /> {i18n.t('skills.header_hooks')}
                            </h3>
                            <button
                                onClick={() => {
                                    setEditingHook({});
                                    setHookForm({ name: '', description: '', hook_type: 'pre_validation', content: '', active: true, category: activeCategory });
                                    setIsHookModalOpen(true);
                                }}
                                className="bg-emerald-600 hover:bg-emerald-500 text-white px-3 py-1.5 rounded-lg text-xs font-bold flex items-center gap-2 transition-colors shadow-[0_0_15px_rgba(16,185,129,0.3)]"
                            >
                                <Plus className="w-3.5 h-3.5" /> Register Hook
                            </button>
                        </div>

                        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 relative z-10">
                            {hooks.filter(h => h.category === activeCategory && (h.name.toLowerCase().includes(searchQuery.toLowerCase()) || h.description.toLowerCase().includes(searchQuery.toLowerCase()))).map(hook => (
                                <div key={hook.name} className={`bg-zinc-950 border border-zinc-800 p-5 rounded-xl transition-all duration-300 hover:border-emerald-500/30 hover:shadow-[0_0_15px_rgba(16,185,129,0.15)] group relative overflow-hidden shadow-sm shadow-black/40 ${hook.active ? '' : 'opacity-60 grayscale'}`}>
                                    <div className="neural-grid opacity-[0.03]" />
                                    <div className="absolute top-4 right-4 flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity z-20">
                                        <button 
                                            onClick={() => {
                                                setEditingHook(hook);
                                                setHookForm(hook);
                                                setIsHookModalOpen(true);
                                            }}
                                            className="text-zinc-500 hover:text-blue-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded transition-colors"
                                        >
                                            <Edit2 className="w-3.5 h-3.5" />
                                        </button>
                                        <button 
                                            onClick={() => handleDeleteHook(hook.name)}
                                            className="text-zinc-500 hover:text-red-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded transition-colors"
                                        >
                                            <Trash2 className="w-3.5 h-3.5" />
                                        </button>
                                    </div>
                                    <div className="relative z-10 flex flex-col h-full">
                                        <div className="flex items-center justify-between mb-3">
                                            <div className="flex items-center gap-2 text-zinc-300 font-bold tracking-wide">
                                                <div className={`w-2 h-2 rounded-full ${hook.active ? 'bg-emerald-500 animate-pulse' : 'bg-zinc-600'} shrink-0`}></div>
                                                <h3 className="font-mono text-sm">{hook.name}</h3>
                                            </div>
                                            <span className="text-[9px] font-bold text-zinc-500 uppercase tracking-widest">{hook.hook_type.replace('_', ' ')}</span>
                                        </div>
                                        <p className="text-zinc-500 text-xs font-mono mb-4 leading-relaxed line-clamp-2 h-8">{hook.description}</p>
                                        <div className="mt-auto pt-4 border-t border-zinc-900 flex items-center justify-between">
                                            <span className={`text-[9px] px-2 py-0.5 rounded border font-bold uppercase ${hook.active ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20' : 'bg-zinc-800 text-zinc-500 border-zinc-700'}`}>
                                                {hook.active ? 'Active' : 'Bypassed'}
                                            </span>
                                            <Shield size={12} className="text-zinc-700" />
                                        </div>
                                    </div>
                                </div>
                            ))}
                            {hooks.filter(h => h.category === activeCategory).length === 0 && (
                                <div className="col-span-full py-12 flex flex-col items-center justify-center border border-dashed border-zinc-800 rounded-2xl bg-zinc-950/20">
                                    <Shield size={32} className="text-zinc-800 mb-4" />
                                    <p className="text-zinc-500 text-sm font-mono">No lifecycle hooks registered in this sector.</p>
                                </div>
                            )}
                        </div>
                    </div>
                ) : ( // activeTab === 'mcp'
                    <div className="space-y-6">
                        <div className="flex justify-between items-center mb-6 sticky top-0 bg-zinc-950/80 backdrop-blur-md pt-2 pb-3 border-b border-zinc-800/50 z-20">
                            <h2 className="text-xl font-bold text-zinc-100 tracking-tight flex items-center gap-3">
                                <FlaskConical className="text-cyan-500" size={20} /> {i18n.t('skills.header_lab')}
                            </h2>
                        </div>

                        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                            {mcpTools.filter(t => t.category === activeCategory && (t.name.toLowerCase().includes(searchQuery.toLowerCase()) || t.description?.toLowerCase().includes(searchQuery.toLowerCase()))).map(tool => (
                                <div key={tool.name} className={`bg-zinc-950/40 border ${tool.isPulsing ? 'border-cyan-500 shadow-[0_0_15px_rgba(6,182,212,0.3)]' : 'border-zinc-800/60'} p-5 rounded-xl group relative overflow-hidden hover:border-cyan-500/30 transition-all duration-300`}>
                                    <div className="absolute top-4 right-4 text-[9px] font-bold tracking-tighter uppercase px-2 py-0.5 rounded border bg-zinc-900 border-zinc-800 text-zinc-500 group-hover:text-cyan-400 group-hover:border-cyan-500/20 transition-colors">
                                        {tool.source}
                                    </div>
                                    <div className="flex items-center gap-3 mb-3">
                                        <div className={`w-2 h-2 rounded-full ${tool.source === 'system' ? 'bg-cyan-500 shadow-[0_0_8px_rgba(6,182,212,0.5)]' : 'bg-zinc-600'} ${tool.isPulsing ? 'animate-ping' : ''}`}></div>
                                        <h3 className="font-mono text-sm text-zinc-100 font-bold">{tool.name}</h3>
                                        {tool.isPulsing && <Activity size={10} className="text-cyan-400 animate-pulse ml-1" />}
                                    </div>
                                    <p className="text-zinc-500 text-xs leading-relaxed mb-4 h-12 overflow-hidden line-clamp-3">
                                        {tool.description}
                                    </p>

                                    {/* Telemetry Metrics */}
                                    <div className="grid grid-cols-3 gap-2 mb-4 bg-black/30 rounded-lg p-2 border border-zinc-900">
                                        <Tooltip content="Total number of times this tool has been invoked by agents." position="top">
                                            <div className="flex flex-col cursor-help">
                                                <div className="text-[10px] text-zinc-500 uppercase tracking-widest font-bold mb-1">{i18n.t('skills.label_invocations')}</div>
                                                <span className="text-[10px] font-mono text-zinc-400">{tool.stats.invocations}</span>
                                            </div>
                                        </Tooltip>
                                        <Tooltip content="Percentage of executions that completed without errors." position="top">
                                            <div className="flex flex-col border-x border-zinc-900 px-2 cursor-help">
                                                <div className="text-[10px] text-zinc-500 uppercase tracking-widest font-bold mb-1">{i18n.t('skills.label_success_rate')}</div>
                                                <span className={`text-[10px] font-mono ${tool.stats.invocations > 0 && (tool.stats.success_count / tool.stats.invocations) < 0.9 ? 'text-amber-500' : 'text-emerald-500'}`}>
                                                    {tool.stats.invocations > 0 ? Math.round((tool.stats.success_count / tool.stats.invocations) * 100) : 0}%
                                                </span>
                                            </div>
                                        </Tooltip>
                                        <Tooltip content="Average round-trip response time for this MCP tool." position="top">
                                            <div className="flex flex-col items-end cursor-help">
                                                <div className="text-[10px] text-zinc-500 uppercase tracking-widest font-bold mb-1 text-right">{i18n.t('skills.label_avg_latency')}</div>
                                                <span className="text-[10px] font-mono text-amber-500">{tool.stats.avg_latency_ms}ms</span>
                                            </div>
                                        </Tooltip>
                                    </div>

                                    <div className="flex items-center justify-between gap-3 mt-auto pt-4 border-t border-zinc-900">
                                        <details className="group/details">
                                            <summary className="text-[9px] font-bold text-zinc-600 cursor-pointer list-none hover:text-zinc-400 transition-colors flex items-center gap-1 uppercase tracking-wider">
                                                <Code size={10} /> {i18n.t('skills.label_schema')}
                                            </summary>
                                            <div className="absolute left-0 right-0 bottom-full mb-2 mx-4 bg-zinc-900/95 backdrop-blur-xl rounded-lg p-4 border border-zinc-800 shadow-2xl z-50 invisible group-open/details:visible opacity-0 group-open/details:opacity-100 transition-all max-h-64 overflow-y-auto custom-scrollbar">
                                                <pre className="text-[10px] text-cyan-300 font-mono">
                                                    {JSON.stringify(tool.input_schema, null, 2)}
                                                </pre>
                                            </div>
                                        </details>

                                         <Tooltip content={i18n.t('agent_manager.btn_assign')} position="top">
                                             <button onClick={() => { setAssignTarget({ type: 'mcp', name: tool.name }); setIsAssignModalOpen(true); }} className="flex items-center gap-2 px-3 py-1.5 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 hover:border-emerald-500/50 rounded-lg text-xs font-bold text-zinc-400 hover:text-emerald-400 transition-all">
                                                 <Users className="w-3.5 h-3.5" /> {i18n.t('agent_manager.btn_assign')}
                                             </button>
                                         </Tooltip>
                                         <Tooltip content={i18n.t('skills.tooltip_test_tool')} position="top">
                                             <button onClick={() => { setLabTool(tool); setLabInput(JSON.stringify(generateSampleInputs(tool.input_schema), null, 2)); setIsLabOpen(true); }} className="flex items-center gap-2 px-3 py-1.5 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 hover:border-cyan-500/50 rounded-lg text-xs font-bold text-zinc-400 hover:text-cyan-400 transition-all">
                                                 <FlaskConical className="w-3.5 h-3.5" /> {i18n.t('skills.btn_test_tool')}
                                             </button>
                                         </Tooltip>
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>
                )}
            </div>

            {/* Skill Edit Modal */}
            {isSkillModalOpen && (
                <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-md flex items-center justify-center p-4">
                    <div className="bg-zinc-950 border border-zinc-800 w-full max-w-2xl rounded-2xl shadow-2xl flex flex-col max-h-[90vh] relative overflow-hidden">
                        <div className="neural-grid opacity-10" />
                        <div className="p-5 border-b border-zinc-800 flex justify-between items-center shrink-0 relative z-10 bg-zinc-950/50">
                            <h2 className="text-lg font-bold text-zinc-100 flex items-center gap-2">
                                <Code className="text-blue-500" /> {editingSkill?.name ? i18n.t('skills.modal_edit_skill') : i18n.t('skills.modal_create_skill')}
                            </h2>
                            <button onClick={() => setIsSkillModalOpen(false)} className="text-zinc-500 hover:text-zinc-300 p-1">✕</button>
                        </div>
                        <div className="p-6 overflow-y-auto space-y-5 custom-scrollbar relative z-10 bg-zinc-950/80">
                            <div>
                                <label className="block text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">{i18n.t('skills.label_skill_name')}</label>
                                <input
                                    type="text"
                                    value={editingSkill.name || ''}
                                    onChange={e => setEditingSkill({ ...editingSkill, name: e.target.value.replace(/\s+/g, '_') })}
                                    className="w-full bg-zinc-900 border border-zinc-800 rounded p-3 text-zinc-200 font-mono text-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 outline-none transition-all placeholder:text-zinc-700"
                                    placeholder="fetch_twitter_data"
                                />
                            </div>
                            <div>
                                <label className="block text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">{i18n.t('skills.label_llm_desc')}</label>
                                <textarea
                                    value={editingSkill.description || ''}
                                    onChange={e => setEditingSkill({ ...editingSkill, description: e.target.value })}
                                    className="w-full bg-zinc-900 border border-zinc-800 rounded p-3 text-zinc-300 focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 outline-none transition-all resize-y min-h-[80px] text-sm placeholder:text-zinc-700"
                                    placeholder="Scrapes a twitter profile..."
                                />
                            </div>
                            <div>
                                <label className="block text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">{i18n.t('skills.label_exec_cmd')}</label>
                                <input
                                    type="text"
                                    value={editingSkill.execution_command || ''}
                                    onChange={e => setEditingSkill({ ...editingSkill, execution_command: e.target.value })}
                                    className="w-full bg-zinc-900 border border-zinc-800 rounded p-3 text-emerald-400 font-mono text-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 outline-none transition-all placeholder:text-zinc-700"
                                    placeholder="python scripts/fetch.py"
                                />
                                <p className="mt-1.5 text-[9px] text-zinc-600 font-mono leading-relaxed">{i18n.t('skills.hint_exec_cmd')}</p>
                            </div>
                            <div>
                                <label className="flex items-center justify-between text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">
                                    <span>{i18n.t('skills.label_params_schema')}</span>
                                    {schemaError && <p className="mt-1 text-[10px] text-red-400 font-bold">{i18n.t('skills.error_invalid_json')}: {schemaError}</p>}
                                </label>
                                <textarea
                                    value={typeof editingSkill.schema === 'string' ? editingSkill.schema : JSON.stringify(editingSkill.schema, null, 2)}
                                    onChange={e => {
                                        try {
                                            const val = JSON.parse(e.target.value);
                                            setEditingSkill({ ...editingSkill, schema: val });
                                            setSchemaError(null);
                                        } catch {
                                            // eslint-disable-next-line @typescript-eslint/no-explicit-any
                                            setEditingSkill({ ...editingSkill, schema: e.target.value as any });
                                            setSchemaError("Invalid JSON");
                                        }
                                    }}
                                    className={`w-full bg-zinc-900 border rounded p-3 font-mono text-xs focus:ring-1 outline-none transition-all resize-y min-h-[150px] custom-scrollbar ${schemaError ? 'border-red-500/50 text-red-400 focus:border-red-500 focus:ring-red-500/50' : 'border-zinc-800 text-zinc-300 focus:border-blue-500 focus:ring-blue-500/50'}`}
                                    spellCheck="false"
                                />
                            </div>
                        </div>
                        <div className="p-5 border-t border-zinc-800 flex justify-end gap-3 shrink-0 relative z-10 bg-zinc-950/90 items-center">
                            {skillSaveError && <div className="text-xs text-red-400 font-mono mr-auto flex items-center gap-1"><AlertTriangle className="w-3.5 h-3.5" /> {skillSaveError}</div>}
                            <button onClick={() => setIsSkillModalOpen(false)} className="px-5 py-2 text-xs font-bold text-zinc-500 hover:text-zinc-100 transition-colors">{i18n.t('skills.btn_cancel')}</button>
                            <button onClick={handleSaveSkill} disabled={isSaving || !!schemaError || !editingSkill.name || !editingSkill.execution_command} className="bg-blue-600 hover:bg-blue-500 text-white px-6 py-2 rounded-lg text-xs font-bold transition-colors shadow-lg shadow-blue-500/20 disabled:opacity-50">{isSaving ? i18n.t('skills.btn_saving') : i18n.t('skills.btn_save_skill')}</button>
                        </div>
                    </div>
                </div>
            )}

            {/* Workflow Edit Modal */}
            {isWfModalOpen && (
                <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-md flex items-center justify-center p-4">
                    <div className="bg-zinc-950 border border-zinc-800 w-full max-w-4xl rounded-2xl shadow-2xl flex flex-col max-h-[90vh] relative overflow-hidden">
                        <div className="neural-grid opacity-10" />
                        <div className="p-5 border-b border-zinc-800 flex justify-between items-center shrink-0 relative z-10 bg-zinc-950/50">
                            <h2 className="text-lg font-bold text-zinc-100 flex items-center gap-2">
                                <FileText className="text-blue-500" /> {editingWf?.name ? i18n.t('skills.modal_edit_workflow') : i18n.t('skills.modal_create_workflow')}
                            </h2>
                            <button onClick={() => setIsWfModalOpen(false)} className="text-zinc-500 hover:text-zinc-300 p-1">✕</button>
                        </div>
                        <div className="p-6 overflow-y-auto space-y-5 custom-scrollbar flex-1 relative z-10 bg-zinc-950/80">
                            <div>
                                <label className="block text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">{i18n.t('skills.label_workflow_name')}</label>
                                <input
                                    type="text"
                                    value={editingWf.name || ''}
                                    onChange={e => setEditingWf({ ...editingWf, name: e.target.value })}
                                    className="w-full bg-zinc-900 border border-zinc-800 rounded p-3 text-blue-300 font-mono text-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 outline-none transition-all placeholder:text-zinc-700"
                                    placeholder="Deep Research Protocol"
                                />
                            </div>
                            <div className="flex-1 flex flex-col h-full min-h-[400px]">
                                <label className="block text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">{i18n.t('skills.label_markdown_content')}</label>
                                <textarea
                                    value={editingWf.content || ''}
                                    onChange={e => setEditingWf({ ...editingWf, content: e.target.value })}
                                    className="flex-1 w-full bg-zinc-900 border border-zinc-800 rounded p-4 text-zinc-300 font-mono text-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 outline-none transition-all resize-none custom-scrollbar placeholder:text-zinc-700"
                                    placeholder="# Protocol Overview\n1. Do X\n2. Do Y"
                                    spellCheck="false"
                                />
                            </div>
                        </div>
                        <div className="p-5 border-t border-zinc-800 flex justify-end gap-3 shrink-0 relative z-10 bg-zinc-950/90 items-center">
                            {wfSaveError && <div className="text-xs text-red-400 font-mono mr-auto flex items-center gap-1"><AlertTriangle className="w-3.5 h-3.5" /> {wfSaveError}</div>}
                            <button onClick={() => setIsWfModalOpen(false)} className="px-5 py-2 text-xs font-bold text-zinc-500 hover:text-zinc-100 transition-colors">{i18n.t('skills.btn_cancel')}</button>
                            <button onClick={handleSaveWf} disabled={isSaving || !editingWf.name || !editingWf.content} className="bg-blue-600 hover:bg-blue-500 text-white px-6 py-2 rounded-lg text-xs font-bold transition-colors shadow-lg shadow-blue-500/20 disabled:opacity-50">{isSaving ? i18n.t('skills.btn_saving') : i18n.t('skills.btn_save_workflow')}</button>
                        </div>
                    </div>
                </div>
            )}
            {/* Tool Lab Modal */}
            {isLabOpen && labTool && (
                <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-200">
                    <div className="bg-zinc-950 border border-zinc-800 w-full max-w-2xl rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
                        <div className="bg-zinc-900/50 p-6 border-b border-zinc-800 flex items-center justify-between">
                            <div className="flex items-center gap-3">
                                <div className="w-10 h-10 rounded-xl bg-cyan-500/10 flex items-center justify-center border border-cyan-500/20 text-cyan-400">
                                    <FlaskConical size={20} />
                                </div>
                                <div>
                                    <h2 className="text-lg font-bold text-zinc-100 font-mono">{labTool.name}</h2>
                                    <p className="text-xs text-zinc-500 uppercase tracking-widest font-bold">{i18n.t('skills.label_manual_execution_lab')}</p>
                                </div>
                            </div>
                            <button onClick={() => setIsLabOpen(false)} className="text-zinc-500 hover:text-zinc-300 transition-colors bg-zinc-800/50 p-2 rounded-lg"><X size={18} /></button>
                        </div>

                        <div className="flex-1 overflow-y-auto p-6 custom-scrollbar space-y-6">
                            <div>
                                <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-1.5">{i18n.t('skills.label_input_args')}</label>
                                <textarea
                                    value={labInput}
                                    onChange={(e) => setLabInput(e.target.value)}
                                    className="w-full h-40 bg-black/50 border border-zinc-800 rounded-xl p-4 font-mono text-xs text-cyan-300 focus:outline-none focus:border-cyan-500/50 transition-colors custom-scrollbar"
                                    placeholder='{ "arg": "value" }'
                                />
                                {labSchemaError && (
                                    <div className="mt-2 text-[10px] text-red-400 font-mono bg-red-400/5 p-2 rounded border border-red-400/10 flex items-center gap-2">
                                        <AlertTriangle size={12} /> {labSchemaError}
                                    </div>
                                )}
                            </div>

                            <div className="flex items-center justify-between">
                                <div className="flex items-center gap-4 text-[10px] font-bold text-zinc-600 uppercase">
                                    <span className="flex items-center gap-1.5"><Gauge size={12} /> {i18n.t('skills.label_live_telemetry')}</span>
                                    <span className="flex items-center gap-1.5"><Shield size={12} /> {i18n.t('skills.label_governance_audit')}</span>
                                </div>
                                <button
                                    onClick={handleRunTool}
                                    disabled={isLabRunning}
                                    className={`flex items-center gap-2 px-6 py-2.5 bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-500 hover:to-blue-500 text-white rounded-xl text-sm font-bold shadow-lg shadow-cyan-500/20 transition-all active:scale-95 disabled:opacity-50 disabled:grayscale`}
                                >
                                    {isLabRunning ? <Activity size={18} className="animate-spin" /> : <Play size={18} fill="currentColor" />}
                                    {isLabRunning ? i18n.t('skills.btn_executing') : i18n.t('skills.btn_run_tool')}
                                </button>
                            </div>

                            {labResult && (
                                <div className="animate-in slide-in-from-bottom-4 duration-300">
                                    <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-1.5">{i18n.t('skills.label_execution_result')}</label>
                                    <div className={`p-4 rounded-xl border font-mono text-xs overflow-x-auto custom-scrollbar ${labResult.status === 'success' ? 'bg-emerald-500/5 border-emerald-500/20 text-emerald-400' : 'bg-red-500/5 border-red-500/20 text-red-400'}`}>
                                        <pre>{JSON.stringify(labResult, null, 2)}</pre>
                                    </div>
                                </div>
                            )}
                        </div>
                    </div>
                </div>
            )}
            {/* Hook Modal */}
            {isHookModalOpen && (
                <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/80 backdrop-blur-md">
                    <div className="bg-zinc-950 border border-zinc-800 w-full max-w-2xl rounded-2xl overflow-hidden shadow-2xl neural-border">
                        <div className="p-6 border-b border-zinc-800 flex items-center justify-between bg-zinc-900/50">
                            <div>
                                <h3 className="text-lg font-bold text-white flex items-center gap-2">
                                    <Shield className="text-emerald-500" size={20} />
                                    {editingHook.name ? 'Edit Lifecycle Hook' : 'Register New Hook'}
                                </h3>
                                <p className="text-xs text-zinc-500">Configure interceptor behavior and validation logic</p>
                            </div>
                            <button onClick={() => setIsHookModalOpen(false)} className="text-zinc-500 hover:text-white">
                                <X size={20} />
                            </button>
                        </div>

                        <div className="p-8 space-y-6 max-h-[70vh] overflow-y-auto custom-scrollbar">
                            <div className="grid grid-cols-2 gap-6">
                                <div className="space-y-2">
                                    <label className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest pl-1">Hook Identifier</label>
                                    <input
                                        type="text"
                                        placeholder="e.g. BudgetGuard_L1"
                                        value={hookForm.name}
                                        onChange={(e) => setHookForm({ ...hookForm, name: e.target.value })}
                                        className="w-full bg-zinc-900 border border-zinc-800 rounded-xl px-4 py-3 text-sm text-zinc-200 focus:outline-none focus:border-emerald-500/50 focus:ring-1 focus:ring-emerald-500/20 transition-all placeholder:text-zinc-700"
                                    />
                                </div>
                                <div className="space-y-2">
                                    <label className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest pl-1">Interceptor Type</label>
                                    <select
                                        value={hookForm.hook_type}
                                        onChange={(e) => setHookForm({ ...hookForm, hook_type: e.target.value })}
                                        className="w-full bg-zinc-900 border border-zinc-800 rounded-xl px-4 py-3 text-sm text-zinc-200 focus:outline-none focus:border-emerald-500/50 transition-all cursor-pointer"
                                    >
                                        <option value="pre_validation">Pre-Validation</option>
                                        <option value="post_analysis">Post-Analysis</option>
                                        <option value="mission_start">Mission Start</option>
                                        <option value="mission_complete">Mission Complete</option>
                                    </select>
                                </div>
                            </div>

                            <div className="space-y-2">
                                <label className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest pl-1">Abstract Functional Description</label>
                                <textarea
                                    rows={3}
                                    placeholder="Briefly describe what this hook intercepts and validates..."
                                    value={hookForm.description}
                                    onChange={(e) => setHookForm({ ...hookForm, description: e.target.value })}
                                    className="w-full bg-zinc-900 border border-zinc-800 rounded-xl px-4 py-3 text-sm text-zinc-200 focus:outline-none focus:border-emerald-500/50 transition-all resize-none placeholder:text-zinc-700"
                                />
                            </div>

                            <div className="space-y-2">
                                <div className="flex items-center justify-between mb-1">
                                    <label className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest pl-1">Validation Payload (JSON/Script)</label>
                                    <div className="text-[9px] text-emerald-500/50 font-mono tracking-tighter">Interception Logic</div>
                                </div>
                                <textarea
                                    rows={8}
                                    placeholder='{ "logic": "block_if_exceeds_budget", "threshold": 10 }'
                                    value={hookForm.content}
                                    onChange={(e) => setHookForm({ ...hookForm, content: e.target.value })}
                                    className="w-full bg-zinc-950/50 border border-zinc-800 rounded-xl px-4 py-4 text-xs font-mono text-emerald-400 focus:outline-none focus:border-emerald-500/30 transition-all placeholder:text-emerald-900/30"
                                />
                            </div>

                            <div className="flex items-center gap-6 p-4 bg-zinc-900/30 rounded-xl border border-zinc-800/50">
                                <div className="flex items-center gap-3">
                                    <button
                                        onClick={() => setHookForm({ ...hookForm, active: !hookForm.active })}
                                        className={`w-10 h-5 rounded-full transition-all relative ${hookForm.active ? 'bg-emerald-600' : 'bg-zinc-800'}`}
                                    >
                                        <div className={`absolute top-1 w-3 h-3 rounded-full bg-white transition-all ${hookForm.active ? 'left-6' : 'left-1'}`} />
                                    </button>
                                    <span className="text-[11px] font-bold text-zinc-400 uppercase tracking-wider">Hook Status: {hookForm.active ? 'Active' : 'Bypassed'}</span>
                                </div>
                                <div className="h-4 w-px bg-zinc-800" />
                                <div className="flex items-center gap-2">
                                    <Shield size={14} className={hookForm.category === 'ai' ? 'text-indigo-500' : 'text-blue-500'} />
                                    <span className="text-[11px] font-bold text-zinc-400 uppercase tracking-wider">Registry: {hookForm.category === 'ai' ? 'AI Sector' : 'User Domain'}</span>
                                </div>
                                {hookSaveError && <div className="text-red-500 text-[10px] ml-auto">{hookSaveError}</div>}
                            </div>
                        </div>

                        <div className="p-6 border-t border-zinc-800 bg-zinc-900/50 flex justify-end gap-3">
                            <button onClick={() => setIsHookModalOpen(false)} className="px-5 py-2.5 text-xs text-zinc-500 hover:text-white font-bold transition-all uppercase tracking-widest">
                                Cancel
                            </button>
                            <button 
                                onClick={handleSaveHook} 
                                disabled={isSaving}
                                className="px-6 py-2.5 bg-emerald-600/20 text-emerald-400 border border-emerald-500/30 rounded-xl text-xs font-bold uppercase tracking-widest hover:bg-emerald-600/30 transition-all shadow-lg hover:shadow-emerald-500/20 disabled:opacity-50"
                            >
                                {isSaving ? 'Processing...' : editingHook.name ? 'Update Registry' : 'Initialize Hook'}
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Assignment Modal */}
            {isAssignModalOpen && assignTarget && (
                <div className="fixed inset-0 z-[60] bg-black/80 backdrop-blur-md flex items-center justify-center p-4">
                    <div className="bg-zinc-950 border border-zinc-800 w-full max-w-md rounded-2xl shadow-2xl flex flex-col relative overflow-hidden">
                        <div className="neural-grid opacity-10" />
                        <div className="p-5 border-b border-zinc-800 flex justify-between items-center shrink-0 relative z-10 bg-zinc-950/50">
                            <h2 className="text-sm font-bold text-zinc-100 flex items-center gap-2 uppercase tracking-wider">
                                <Users className="text-emerald-500" size={16} /> {i18n.t('agent_manager.modal_assign_title', { type: assignTarget.type.toUpperCase(), name: assignTarget.name })}
                            </h2>
                            <button onClick={() => setIsAssignModalOpen(false)} className="text-zinc-500 hover:text-zinc-300 p-1">✕</button>
                        </div>
                        <div className="p-6 space-y-4 relative z-10 bg-zinc-950/80">
                            <p className="text-xs text-zinc-500 font-mono leading-relaxed">
                                {i18n.t('agent_manager.label_select_agents')}
                            </p>
                            <div className="space-y-2 max-h-64 overflow-y-auto custom-scrollbar pr-2">
                                {agents.map(agent => {
                                    const isAssigned = 
                                        (assignTarget.type === 'skill' && (agent.skills || []).includes(assignTarget.name)) ||
                                        (assignTarget.type === 'workflow' && (agent.workflows || []).includes(assignTarget.name)) ||
                                        (assignTarget.type === 'mcp' && (agent.mcpTools || []).includes(assignTarget.name));
                                    
                                    return (
                                        <button
                                            key={agent.id}
                                            onClick={() => handleToggleAssignment(agent.id)}
                                            className={`w-full flex items-center justify-between p-3 rounded-xl border transition-all ${
                                                isAssigned 
                                                    ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400 shadow-[0_0_10px_rgba(16,185,129,0.1)]' 
                                                    : 'bg-zinc-900/50 border-zinc-800 text-zinc-400 hover:border-zinc-700'
                                            }`}
                                        >
                                            <div className="flex items-center gap-3">
                                                <div 
                                                    className="w-8 h-8 rounded-lg flex items-center justify-center font-bold text-sm shrink-0"
                                                    style={{ 
                                                        backgroundColor: agent.themeColor ? `${agent.themeColor}20` : '#27272a',
                                                        color: agent.themeColor || '#71717a',
                                                        border: `1px solid ${agent.themeColor ? `${agent.themeColor}40` : '#3f3f46'}`
                                                    }}
                                                >
                                                    {agent.name[0]}
                                                </div>
                                                <div className="text-left">
                                                    <div className="text-xs font-bold truncate max-w-[180px]">{agent.name}</div>
                                                    <div className="text-[10px] opacity-50 font-mono uppercase">{agent.role}</div>
                                                </div>
                                            </div>
                                            {isAssigned && <Check size={14} className="text-emerald-500" />}
                                        </button>
                                    );
                                })}
                            </div>
                        </div>
                        <div className="p-6 border-t border-zinc-800 bg-zinc-900/50 flex justify-end">
                            <button 
                                onClick={() => setIsAssignModalOpen(false)} 
                                className="px-6 py-2 bg-zinc-800 hover:bg-zinc-700 text-white rounded-lg text-xs font-bold uppercase tracking-widest transition-all"
                            >
                                Done
                            </button>
                        </div>
                    </div>
                </div>
            )}
            {/* Import Preview Modal */}
            <ImportPreviewModal 
                isOpen={isImportModalOpen}
                type={importType}
                data={importData}
                preview={importPreview}
                onClose={() => setIsImportModalOpen(false)}
                onConfirm={handleConfirmImport}
            />
        </div>
    );
}

