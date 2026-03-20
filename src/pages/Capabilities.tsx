import { useState, useEffect } from 'react';
import { Settings, Plus, Trash2, Edit2, Code, FileText, AlertTriangle, Activity, Terminal, Shield, FlaskConical, Play, Gauge, X } from 'lucide-react';
import { useCapabilitiesStore, type SkillDefinition, type WorkflowDefinition, type McpToolHubDefinition } from '../stores/capabilitiesStore';
import { TwEmptyState, Tooltip } from '../components/ui';
import { i18n } from '../i18n';

import { TadpoleOSService } from '../services/tadpoleosService';

export default function Capabilities() {
    const { skills, workflows, mcpTools, isLoading, error, fetchCapabilities, fetchMcpTools, saveSkill, deleteSkill, saveWorkflow, deleteWorkflow } = useCapabilitiesStore();
    const [activeTab, setActiveTab] = useState<'skills' | 'workflows' | 'hooks' | 'mcp'>('skills');

    // Skill Modal State
    const [isSkillModalOpen, setIsSkillModalOpen] = useState(false);
    const [editingSkill, setEditingSkill] = useState<Partial<SkillDefinition>>({});
    const [skillSaveError, setSkillSaveError] = useState<string | null>(null);
    const [schemaError, setSchemaError] = useState<string | null>(null);

    // Workflow Modal State
    const [isWfModalOpen, setIsWfModalOpen] = useState(false);
    const [editingWf, setEditingWf] = useState<Partial<WorkflowDefinition>>({});
    const [wfSaveError, setWfSaveError] = useState<string | null>(null);

    // Shared State
    const [isSaving, setIsSaving] = useState(false);

    // Tool Lab State
    const [labTool, setLabTool] = useState<McpToolHubDefinition | null>(null);
    const [isLabOpen, setIsLabOpen] = useState(false);
    const [labInput, setLabInput] = useState<string>('{}');
    const [labResult, setLabResult] = useState<Record<string, unknown> | null>(null);
    const [isLabRunning, setIsLabRunning] = useState(false);
    const [labSchemaError, setLabSchemaError] = useState<string | null>(null);

    useEffect(() => {
        fetchCapabilities();
        fetchMcpTools();
    }, [fetchCapabilities, fetchMcpTools]);

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

    const generateSampleInputs = (schema: { properties?: Record<string, { type: string }> }) => {
        const inputs: Record<string, unknown> = {};
        if (schema.properties) {
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
            await saveSkill(editingSkill as SkillDefinition);
            setIsSkillModalOpen(false);
        } catch (e: unknown) {
            setSkillSaveError(e instanceof Error ? e.message : 'Unknown error');
        } finally {
            setIsSaving(false);
        }
    };

    const handleSaveWf = async () => {
        if (!editingWf.name || !editingWf.content) return;
        setWfSaveError(null);
        setIsSaving(true);
        try {
            await saveWorkflow(editingWf as WorkflowDefinition);
            setIsWfModalOpen(false);
        } catch (e: unknown) {
            setWfSaveError(e instanceof Error ? e.message : 'Unknown error');
        } finally {
            setIsSaving(false);
        }
    };

    return (
        <div className="flex flex-col h-full gap-6">
            {/* Standardized Module Header */}
            <div className="flex items-center justify-between border-b border-zinc-900 pb-2 px-1 shrink-0">
                <div>
                    <h2 className="text-xl font-bold flex items-center gap-2 text-zinc-100">
                        <Tooltip content={i18n.t('capabilities.tooltip_main')} position="right">
                            <Settings className="text-blue-500 cursor-help" />
                        </Tooltip>
                        {i18n.t('capabilities.title')}
                    </h2>
                    <div className="mt-1">
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
                <Tooltip content={i18n.t('capabilities.tooltip_skills')} position="bottom">
                    <button
                        onClick={() => setActiveTab('skills')}
                        className={`flex items-center gap-2 px-6 py-3 font-medium transition-colors border-b-2 ${activeTab === 'skills' ? 'border-blue-500 text-blue-400' : 'border-transparent text-zinc-500 hover:text-zinc-300'
                            }`}
                    >
                        <Code className="w-4 h-4" /> {i18n.t('capabilities.tab_skills')}
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('capabilities.tooltip_workflows')} position="bottom">
                    <button
                        onClick={() => setActiveTab('workflows')}
                        className={`flex items-center gap-2 px-6 py-3 font-medium transition-colors border-b-2 ${activeTab === 'workflows' ? 'border-blue-500 text-blue-400' : 'border-transparent text-zinc-500 hover:text-zinc-300'
                            }`}
                    >
                        <FileText className="w-4 h-4" /> {i18n.t('capabilities.tab_workflows')}
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('capabilities.tooltip_hooks')} position="bottom">
                    <button
                        onClick={() => setActiveTab('hooks')}
                        className={`flex items-center gap-2 px-6 py-3 font-medium transition-colors border-b-2 ${activeTab === 'hooks' ? 'border-blue-500 text-blue-400' : 'border-transparent text-zinc-500 hover:text-zinc-300'
                            }`}
                    >
                        <Shield className="w-4 h-4" /> {i18n.t('capabilities.tab_hooks')}
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('capabilities.tooltip_mcp')} position="bottom">
                    <button
                        onClick={() => setActiveTab('mcp')}
                        className={`flex items-center gap-2 px-6 py-3 font-medium transition-colors border-b-2 ${activeTab === 'mcp' ? 'border-cyan-500 text-cyan-400' : 'border-transparent text-zinc-500 hover:text-zinc-300'
                            }`}
                    >
                        <Terminal className="w-4 h-4" /> {i18n.t('capabilities.tab_mcp')}
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
                                <Activity size={12} className="text-blue-500" /> {i18n.t('capabilities.header_execution')}
                            </h3>
                            <Tooltip content={i18n.t('capabilities.tooltip_new_skill')} position="bottom">
                                <button
                                    onClick={() => {
                                        setEditingSkill({ schema: { type: "object", properties: {} } });
                                        setSkillSaveError(null);
                                        setSchemaError(null);
                                        setIsSkillModalOpen(true);
                                    }}
                                    className="bg-blue-600 hover:bg-blue-500 text-white px-3 py-1.5 rounded-lg text-xs font-bold flex items-center gap-2 transition-colors shadow-[0_0_15px_rgba(37,99,235,0.3)]"
                                >
                                    <Plus className="w-3.5 h-3.5" /> {i18n.t('capabilities.btn_new_skill')}
                                </button>
                            </Tooltip>
                        </div>

                        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 relative z-10">
                            {skills.map(skill => (
                                <div key={skill.name} className="bg-zinc-950 border border-zinc-800 p-5 rounded-xl transition-all duration-300 hover:border-emerald-500/30 hover:shadow-[0_0_15px_rgba(16,185,129,0.15)] group relative overflow-hidden shadow-sm">
                                    <div className="neural-grid opacity-[0.03]" />
                                    <div className="absolute top-4 right-4 flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity z-20">
                                        <Tooltip content={i18n.t('capabilities.tooltip_edit_skill')} position="top">
                                            <button onClick={() => { setEditingSkill(skill); setSkillSaveError(null); setSchemaError(null); setIsSkillModalOpen(true); }} className="text-zinc-500 hover:text-blue-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded"><Edit2 className="w-3.5 h-3.5" /></button>
                                        </Tooltip>
                                        <Tooltip content={i18n.t('capabilities.tooltip_delete_skill')} position="top">
                                            <button onClick={() => deleteSkill(skill.name)} className="text-zinc-500 hover:text-red-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded"><Trash2 className="w-3.5 h-3.5" /></button>
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
                             {skills.length === 0 && <TwEmptyState title={i18n.t('capabilities.empty_skills_title')} description={i18n.t('capabilities.empty_skills_desc')} />}
                        </div>
                    </div>
                ) : activeTab === 'workflows' ? (
                    <div className="space-y-6">
                        <div className="flex justify-between items-center mb-6 sticky top-0 bg-zinc-950/80 backdrop-blur-md pt-2 pb-3 border-b border-zinc-800/50 z-20">
                            <h3 className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                                <Activity size={12} className="text-blue-500" /> {i18n.t('capabilities.header_guiding')}
                            </h3>
                            <Tooltip content={i18n.t('capabilities.tooltip_new_workflow')} position="bottom">
                                <button
                                    onClick={() => {
                                        setEditingWf({});
                                        setWfSaveError(null);
                                        setIsWfModalOpen(true);
                                    }}
                                    className="bg-blue-600 hover:bg-blue-500 text-white px-3 py-1.5 rounded-lg text-xs font-bold flex items-center gap-2 transition-colors shadow-[0_0_15px_rgba(37,99,235,0.3)]"
                                >
                                    <Plus className="w-3.5 h-3.5" /> {i18n.t('capabilities.btn_new_workflow')}
                                </button>
                            </Tooltip>
                        </div>

                        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 relative z-10">
                            {workflows.map(wf => (
                                <div key={wf.name} className="bg-zinc-950 border border-zinc-800 p-5 rounded-xl transition-all duration-300 hover:border-amber-500/30 hover:shadow-[0_0_15px_rgba(245,158,11,0.15)] group relative overflow-hidden shadow-sm">
                                    <div className="neural-grid opacity-[0.03]" />
                                    <div className="absolute top-4 right-4 flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity z-20">
                                        <Tooltip content={i18n.t('capabilities.tooltip_edit_workflow')} position="top">
                                            <button onClick={() => { setEditingWf(wf); setWfSaveError(null); setIsWfModalOpen(true); }} className="text-zinc-500 hover:text-blue-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded"><Edit2 className="w-3.5 h-3.5" /></button>
                                        </Tooltip>
                                        <Tooltip content={i18n.t('capabilities.tooltip_delete_workflow')} position="top">
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
                            {workflows.length === 0 && <TwEmptyState title={i18n.t('capabilities.empty_workflows_title')} description={i18n.t('capabilities.empty_workflows_desc')} />}
                        </div>
                    </div>
                ) : activeTab === 'hooks' ? (
                    <div className="space-y-6">
                        <div className="flex justify-between items-center mb-6 sticky top-0 bg-zinc-950/80 backdrop-blur-md pt-2 pb-3 border-b border-zinc-800/50 z-20">
                            <h3 className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                                <Shield size={12} className="text-emerald-500" /> {i18n.t('capabilities.header_hooks')}
                            </h3>
                        </div>

                        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                            <div className="bg-zinc-900/50 border border-zinc-800 p-5 rounded-xl group relative overflow-hidden">
                                 <div className="absolute top-4 right-4 text-emerald-500/50 group-hover:text-emerald-400 transition-colors uppercase text-[10px] font-bold tracking-tighter">{i18n.t('capabilities.hook_pre_validation')}</div>
                                <h3 className="font-mono text-sm text-zinc-100 mb-2">audit_governance_v1</h3>
                                <p className="text-zinc-500 text-xs font-mono mb-4">Ensures all tool calls comply with core safety directives before execution. Blocks unauthorized parameter modification.</p>
                                <div className="flex items-center gap-2">
                                    <span className="text-[9px] bg-emerald-500/10 text-emerald-400 px-2 py-0.5 rounded border border-emerald-500/20 uppercase font-bold">{i18n.t('capabilities.status_active')}</span>
                                    <span className="text-[9px] bg-zinc-800 text-zinc-400 px-2 py-0.5 rounded border border-zinc-700/50 uppercase font-bold">{i18n.t('capabilities.type_security')}</span>
                                </div>
                            </div>
                            <div className="bg-zinc-900/50 border border-zinc-800 p-5 rounded-xl group relative overflow-hidden">
                                 <div className="absolute top-4 right-4 text-blue-500/50 group-hover:text-blue-400 transition-colors uppercase text-[10px] font-bold tracking-tighter">{i18n.t('capabilities.hook_post_analysis')}</div>
                                <h3 className="font-mono text-sm text-zinc-100 mb-2">memory_sync_v2</h3>
                                <p className="text-zinc-500 text-xs font-mono mb-4">Extracts key insights from tool outputs and commits them to long-term persistent memory ledgers.</p>
                                <div className="flex items-center gap-2">
                                    <span className="text-[9px] bg-emerald-500/10 text-emerald-400 px-2 py-0.5 rounded border border-emerald-500/20 uppercase font-bold">Status: Active</span>
                                    <span className="text-[9px] bg-zinc-800 text-zinc-400 px-2 py-0.5 rounded border border-zinc-700/50 uppercase font-bold">Type: Cognitive</span>
                                </div>
                            </div>
                        </div>
                    </div>
                ) : ( // activeTab === 'mcp'
                    <div className="space-y-6">
                        <div className="flex justify-between items-center mb-6 sticky top-0 bg-zinc-950/80 backdrop-blur-md pt-2 pb-3 border-b border-zinc-800/50 z-20">
                            <h2 className="text-xl font-bold text-zinc-100 tracking-tight flex items-center gap-3">
                                <FlaskConical className="text-cyan-500" size={20} /> {i18n.t('capabilities.header_lab')}
                            </h2>
                        </div>

                        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                            {mcpTools.map(tool => (
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
                                                <div className="text-[10px] text-zinc-500 uppercase tracking-widest font-bold mb-1">{i18n.t('capabilities.label_invocations')}</div>
                                                <span className="text-[10px] font-mono text-zinc-400">{tool.stats.invocations}</span>
                                            </div>
                                        </Tooltip>
                                        <Tooltip content="Percentage of executions that completed without errors." position="top">
                                            <div className="flex flex-col border-x border-zinc-900 px-2 cursor-help">
                                                <div className="text-[10px] text-zinc-500 uppercase tracking-widest font-bold mb-1">{i18n.t('capabilities.label_success_rate')}</div>
                                                <span className={`text-[10px] font-mono ${tool.stats.invocations > 0 && (tool.stats.success_count / tool.stats.invocations) < 0.9 ? 'text-amber-500' : 'text-emerald-500'}`}>
                                                    {tool.stats.invocations > 0 ? Math.round((tool.stats.success_count / tool.stats.invocations) * 100) : 0}%
                                                </span>
                                            </div>
                                        </Tooltip>
                                        <Tooltip content="Average round-trip response time for this MCP tool." position="top">
                                            <div className="flex flex-col items-end cursor-help">
                                                <div className="text-[10px] text-zinc-500 uppercase tracking-widest font-bold mb-1 text-right">{i18n.t('capabilities.label_avg_latency')}</div>
                                                <span className="text-[10px] font-mono text-amber-500">{tool.stats.avg_latency_ms}ms</span>
                                            </div>
                                        </Tooltip>
                                    </div>

                                    <div className="flex items-center justify-between gap-3 mt-auto pt-4 border-t border-zinc-900">
                                        <details className="group/details">
                                            <summary className="text-[9px] font-bold text-zinc-600 cursor-pointer list-none hover:text-zinc-400 transition-colors flex items-center gap-1 uppercase tracking-wider">
                                                <Code size={10} /> Schema
                                            </summary>
                                            <div className="absolute left-0 right-0 bottom-full mb-2 mx-4 bg-zinc-900/95 backdrop-blur-xl rounded-lg p-4 border border-zinc-800 shadow-2xl z-50 invisible group-open/details:visible opacity-0 group-open/details:opacity-100 transition-all max-h-64 overflow-y-auto custom-scrollbar">
                                                <pre className="text-[10px] text-cyan-300 font-mono">
                                                    {JSON.stringify(tool.input_schema, null, 2)}
                                                </pre>
                                            </div>
                                        </details>

                                         <Tooltip content={i18n.t('capabilities.tooltip_test_tool')} position="top">
                                             <button onClick={() => { setLabTool(tool); setLabInput(JSON.stringify(generateSampleInputs(tool.input_schema), null, 2)); setIsLabOpen(true); }} className="flex items-center gap-2 px-3 py-1.5 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 hover:border-cyan-500/50 rounded-lg text-xs font-bold text-zinc-400 hover:text-cyan-400 transition-all">
                                                 <FlaskConical className="w-3.5 h-3.5" /> {i18n.t('capabilities.btn_test_tool')}
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
                                <Code className="text-blue-500" /> {editingSkill?.name ? i18n.t('capabilities.modal_edit_skill') : i18n.t('capabilities.modal_create_skill')}
                            </h2>
                            <button onClick={() => setIsSkillModalOpen(false)} className="text-zinc-500 hover:text-zinc-300 p-1">✕</button>
                        </div>
                        <div className="p-6 overflow-y-auto space-y-5 custom-scrollbar relative z-10 bg-zinc-950/80">
                            <div>
                                <label className="block text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">{i18n.t('capabilities.label_skill_name')}</label>
                                <input
                                    type="text"
                                    value={editingSkill.name || ''}
                                    onChange={e => setEditingSkill({ ...editingSkill, name: e.target.value.replace(/\s+/g, '_') })}
                                    className="w-full bg-zinc-900 border border-zinc-800 rounded p-3 text-zinc-200 font-mono text-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 outline-none transition-all placeholder:text-zinc-700"
                                    placeholder="fetch_twitter_data"
                                />
                            </div>
                            <div>
                                <label className="block text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">{i18n.t('capabilities.label_llm_desc')}</label>
                                <textarea
                                    value={editingSkill.description || ''}
                                    onChange={e => setEditingSkill({ ...editingSkill, description: e.target.value })}
                                    className="w-full bg-zinc-900 border border-zinc-800 rounded p-3 text-zinc-300 focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 outline-none transition-all resize-y min-h-[80px] text-sm placeholder:text-zinc-700"
                                    placeholder="Scrapes a twitter profile..."
                                />
                            </div>
                            <div>
                                <label className="block text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">{i18n.t('capabilities.label_exec_cmd')}</label>
                                <input
                                    type="text"
                                    value={editingSkill.execution_command || ''}
                                    onChange={e => setEditingSkill({ ...editingSkill, execution_command: e.target.value })}
                                    className="w-full bg-zinc-900 border border-zinc-800 rounded p-3 text-emerald-400 font-mono text-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 outline-none transition-all placeholder:text-zinc-700"
                                    placeholder="python scripts/fetch.py"
                                />
                                <p className="mt-1.5 text-[9px] text-zinc-600 font-mono leading-relaxed">{i18n.t('capabilities.hint_exec_cmd')}</p>
                            </div>
                            <div>
                                <label className="flex items-center justify-between text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">
                                    <span>{i18n.t('capabilities.label_params_schema')}</span>
                                    {schemaError && <p className="mt-1 text-[10px] text-red-400 font-bold">{i18n.t('capabilities.error_invalid_json')}: {schemaError}</p>}
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
                            <button onClick={() => setIsSkillModalOpen(false)} className="px-5 py-2 text-xs font-bold text-zinc-500 hover:text-zinc-100 transition-colors">{i18n.t('capabilities.btn_cancel')}</button>
                            <button onClick={handleSaveSkill} disabled={isSaving || !!schemaError || !editingSkill.name || !editingSkill.execution_command} className="bg-blue-600 hover:bg-blue-500 text-white px-6 py-2 rounded-lg text-xs font-bold transition-colors shadow-lg shadow-blue-500/20 disabled:opacity-50">{isSaving ? i18n.t('capabilities.btn_saving') : i18n.t('capabilities.btn_save_skill')}</button>
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
                                <FileText className="text-blue-500" /> {editingWf?.name ? i18n.t('capabilities.modal_edit_workflow') : i18n.t('capabilities.modal_create_workflow')}
                            </h2>
                            <button onClick={() => setIsWfModalOpen(false)} className="text-zinc-500 hover:text-zinc-300 p-1">✕</button>
                        </div>
                        <div className="p-6 overflow-y-auto space-y-5 custom-scrollbar flex-1 relative z-10 bg-zinc-950/80">
                            <div>
                                <label className="block text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">{i18n.t('capabilities.label_workflow_name')}</label>
                                <input
                                    type="text"
                                    value={editingWf.name || ''}
                                    onChange={e => setEditingWf({ ...editingWf, name: e.target.value })}
                                    className="w-full bg-zinc-900 border border-zinc-800 rounded p-3 text-blue-300 font-mono text-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 outline-none transition-all placeholder:text-zinc-700"
                                    placeholder="Deep Research Protocol"
                                />
                            </div>
                            <div className="flex-1 flex flex-col h-full min-h-[400px]">
                                <label className="block text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">{i18n.t('capabilities.label_markdown_content')}</label>
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
                            <button onClick={() => setIsWfModalOpen(false)} className="px-5 py-2 text-xs font-bold text-zinc-500 hover:text-zinc-100 transition-colors">{i18n.t('capabilities.btn_cancel')}</button>
                            <button onClick={handleSaveWf} disabled={isSaving || !editingWf.name || !editingWf.content} className="bg-blue-600 hover:bg-blue-500 text-white px-6 py-2 rounded-lg text-xs font-bold transition-colors shadow-lg shadow-blue-500/20 disabled:opacity-50">{isSaving ? i18n.t('capabilities.btn_saving') : i18n.t('capabilities.btn_save_workflow')}</button>
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
                                    <p className="text-xs text-zinc-500 uppercase tracking-widest font-bold">{i18n.t('capabilities.label_manual_execution_lab')}</p>
                                </div>
                            </div>
                            <button onClick={() => setIsLabOpen(false)} className="text-zinc-500 hover:text-zinc-300 transition-colors bg-zinc-800/50 p-2 rounded-lg"><X size={18} /></button>
                        </div>

                        <div className="flex-1 overflow-y-auto p-6 custom-scrollbar space-y-6">
                            <div>
                                <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-1.5">{i18n.t('capabilities.label_input_args')}</label>
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
                                    <span className="flex items-center gap-1.5"><Gauge size={12} /> {i18n.t('capabilities.label_live_telemetry')}</span>
                                    <span className="flex items-center gap-1.5"><Shield size={12} /> {i18n.t('capabilities.label_governance_audit')}</span>
                                </div>
                                <button
                                    onClick={handleRunTool}
                                    disabled={isLabRunning}
                                    className={`flex items-center gap-2 px-6 py-2.5 bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-500 hover:to-blue-500 text-white rounded-xl text-sm font-bold shadow-lg shadow-cyan-500/20 transition-all active:scale-95 disabled:opacity-50 disabled:grayscale`}
                                >
                                    {isLabRunning ? <Activity size={18} className="animate-spin" /> : <Play size={18} fill="currentColor" />}
                                    {isLabRunning ? i18n.t('capabilities.btn_executing') : i18n.t('capabilities.btn_run_tool')}
                                </button>
                            </div>

                            {labResult && (
                                <div className="animate-in slide-in-from-bottom-4 duration-300">
                                    <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-1.5">{i18n.t('capabilities.label_execution_result')}</label>
                                    <div className={`p-4 rounded-xl border font-mono text-xs overflow-x-auto custom-scrollbar ${labResult.status === 'success' ? 'bg-emerald-500/5 border-emerald-500/20 text-emerald-400' : 'bg-red-500/5 border-red-500/20 text-red-400'}`}>
                                        <pre>{JSON.stringify(labResult, null, 2)}</pre>
                                    </div>
                                </div>
                            )}
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}

