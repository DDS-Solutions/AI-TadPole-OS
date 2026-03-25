import { useState, useEffect } from 'react';
import { useSkillStore, type SkillDefinition, type WorkflowDefinition, type McpToolHubDefinition, type HookDefinition } from '../stores/skillStore';
import { useAgentStore } from '../stores/agentStore';
import { TadpoleOSService } from '../services/tadpoleosService';
import { AgentApiService } from '../services/AgentApiService';
import { ImportPreviewModal } from '../components/ImportPreviewModal';
import type { Agent } from '../types';

import {
    SkillHeader,
    SkillTabs,
    SkillList,
    WorkflowList,
    HookList,
    McpToolList,
    SkillEditModal,
    WorkflowEditModal,
    HookModal,
    McpLabModal,
    AssignmentModal
} from '../components/skills';

export default function Skills() {
    const { 
        scripts, workflows, mcpTools, manifests, hooks, isLoading, error, 
        fetchSkills, fetchMcpTools, saveSkillScript, deleteSkillScript, 
        saveWorkflow, deleteWorkflow, saveHook, deleteHook 
    } = useSkillStore();
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
                ? currentSkills.filter((s) => s !== name)
                : [...currentSkills, name];
        } else if (type === 'workflow') {
            const currentWorkflows = agent.workflows || [];
            updates.workflows = currentWorkflows.includes(name)
                ? currentWorkflows.filter((w: string) => w !== name)
                : [...currentWorkflows, name];
        } else if (type === 'mcp') {
            const currentMcp = agent.mcpTools || [];
            updates.mcpTools = currentMcp.includes(name)
                ? currentMcp.filter((t: string) => t !== name)
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

    const userRegistryCount = scripts.filter(s => s.category === 'user').length + workflows.filter(w => w.category === 'user').length;
    const aiServicesCount = manifests.filter(m => m.category === 'ai').length + workflows.filter(w => w.category === 'ai').length;

    return (
        <div className="flex flex-col h-full gap-6">
            <SkillHeader 
                activeCategory={activeCategory}
                setActiveCategory={setActiveCategory}
                searchQuery={searchQuery}
                setSearchQuery={setSearchQuery}
                handleImportClick={handleImportClick}
                isSaving={isSaving}
                userRegistryCount={userRegistryCount}
                aiServicesCount={aiServicesCount}
            />

            {error && (
                <div className="bg-red-900/10 border border-red-500/20 p-4 rounded-xl text-red-400 mb-2 flex items-start gap-3">
                    <p>{error}</p>
                </div>
            )}

            <SkillTabs activeTab={activeTab} setActiveTab={setActiveTab} />

            <div className="flex-1 overflow-y-auto min-h-0 custom-scrollbar pr-2 pb-6">
                {isLoading ? (
                    <div className="flex justify-center p-12"><div className="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"></div></div>
                ) : activeTab === 'skills' ? (
                    <SkillList 
                        skills={scripts}
                        searchQuery={searchQuery}
                        activeCategory={activeCategory}
                        onNewSkill={() => {
                            setEditingSkill({ schema: { type: "object", properties: {} } });
                            setSkillSaveError(null);
                            setSchemaError(null);
                            setIsSkillModalOpen(true);
                        }}
                        onEditSkill={(skill) => {
                            setEditingSkill(skill);
                            setSkillSaveError(null);
                            setSchemaError(null);
                            setIsSkillModalOpen(true);
                        }}
                        onAssignSkill={(name) => {
                            setAssignTarget({ type: 'skill', name });
                            setIsAssignModalOpen(true);
                        }}
                        onDeleteSkill={deleteSkillScript}
                    />
                ) : activeTab === 'workflows' ? (
                    <WorkflowList 
                        workflows={workflows}
                        searchQuery={searchQuery}
                        activeCategory={activeCategory}
                        onNewWorkflow={() => {
                            setEditingWf({});
                            setWfSaveError(null);
                            setIsWfModalOpen(true);
                        }}
                        onEditWorkflow={(wf) => {
                            setEditingWf(wf);
                            setWfSaveError(null);
                            setIsWfModalOpen(true);
                        }}
                        onAssignWorkflow={(name) => {
                            setAssignTarget({ type: 'workflow', name });
                            setIsAssignModalOpen(true);
                        }}
                        onDeleteWorkflow={deleteWorkflow}
                    />
                ) : activeTab === 'hooks' ? (
                    <HookList 
                        hooks={hooks}
                        searchQuery={searchQuery}
                        activeCategory={activeCategory}
                        onNewHook={() => {
                            setEditingHook({});
                            setHookForm({ name: '', description: '', hook_type: 'pre_validation', content: '', active: true, category: activeCategory });
                            setIsHookModalOpen(true);
                        }}
                        onEditHook={(hook) => {
                            setEditingHook(hook);
                            setHookForm(hook);
                            setIsHookModalOpen(true);
                        }}
                        onDeleteHook={handleDeleteHook}
                    />
                ) : (
                    <McpToolList 
                        mcpTools={mcpTools}
                        searchQuery={searchQuery}
                        activeCategory={activeCategory}
                        onAssignTool={(name) => {
                            setAssignTarget({ type: 'mcp', name });
                            setIsAssignModalOpen(true);
                        }}
                        onTestTool={(tool) => {
                            setLabTool(tool);
                            setLabInput(JSON.stringify(generateSampleInputs(tool.input_schema), null, 2));
                            setIsLabOpen(true);
                        }}
                    />
                )}
            </div>

            <SkillEditModal 
                isOpen={isSkillModalOpen}
                onClose={() => setIsSkillModalOpen(false)}
                editingSkill={editingSkill}
                setEditingSkill={setEditingSkill}
                schemaError={schemaError}
                setSchemaError={setSchemaError}
                skillSaveError={skillSaveError}
                isSaving={isSaving}
                onSave={handleSaveSkill}
            />

            <WorkflowEditModal 
                isOpen={isWfModalOpen}
                onClose={() => setIsWfModalOpen(false)}
                editingWf={editingWf}
                setEditingWf={setEditingWf}
                wfSaveError={wfSaveError}
                isSaving={isSaving}
                onSave={handleSaveWf}
            />

            <HookModal 
                isOpen={isHookModalOpen}
                onClose={() => setIsHookModalOpen(false)}
                editingHook={editingHook}
                hookForm={hookForm}
                setHookForm={setHookForm}
                hookSaveError={hookSaveError}
                isSaving={isSaving}
                onSave={handleSaveHook}
            />

            <McpLabModal 
                isOpen={isLabOpen}
                onClose={() => setIsLabOpen(false)}
                labTool={labTool}
                labInput={labInput}
                setLabInput={setLabInput}
                labSchemaError={labSchemaError}
                isLabRunning={isLabRunning}
                handleRunTool={handleRunTool}
                labResult={labResult}
            />

            <AssignmentModal 
                isOpen={isAssignModalOpen}
                onClose={() => setIsAssignModalOpen(false)}
                assignTarget={assignTarget}
                agents={agents}
                onToggleAssignment={handleToggleAssignment}
            />

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
