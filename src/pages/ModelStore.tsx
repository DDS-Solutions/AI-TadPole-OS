import React, { useState, useEffect } from 'react';
import { Store, Download, Cpu, Activity, Shield, Zap, Info, ChevronRight, HardDrive } from 'lucide-react';
import { TadpoleOSService } from '../services/tadpoleosService';
import type { StoreModel, SwarmNode } from '../services/tadpoleosService';
import { Tooltip } from '../components/ui';
import { EventBus } from '../services/eventBus';
import { i18n } from '../i18n';





export default function ModelStore(): React.ReactElement {
    const [catalog, setCatalog] = useState<StoreModel[]>([]);
    const [nodes, setNodes] = useState<SwarmNode[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [isInstalling, setIsInstalling] = useState<string | null>(null);

    useEffect(() => {
        const loadData = async () => {
            try {
                const [catalogData, nodesData] = await Promise.all([
                    TadpoleOSService.getModelCatalog(),
                    TadpoleOSService.getNodes()
                ]);
                setCatalog(catalogData);
                setNodes(nodesData);
            } catch (error) {
                console.error("Failed to load Model Store data", error);
                EventBus.emit({
                    source: 'System',
                    text: 'Failed to synchronize with Model Store catalog.',
                    severity: 'error'
                });
            } finally {
                setIsLoading(false);
            }
        };
        loadData();
    }, []);

    const handleInstall = async (modelId: string, nodeId: string) => {
        setIsInstalling(modelId);
        try {
            await TadpoleOSService.pullModel(modelId, nodeId);
            EventBus.emit({
                source: 'System',
                text: `Model pull sequence initiated: ${modelId} -> ${nodes.find(n => n.id === nodeId)?.name}`,
                severity: 'info'
            });
        } catch (error) {
            EventBus.emit({
                source: 'System',
                text: `Installation failed: ${error instanceof Error ? error.message : 'Network error'}`,
                severity: 'error'
            });
        } finally {
            setIsInstalling(null);
        }
    };

    if (isLoading) {
        return (
            <div className="flex flex-col items-center justify-center h-full space-y-4">
                <Activity className="animate-spin text-blue-500" size={48} />
                <p className="text-sm font-bold text-zinc-500 uppercase tracking-widest">{i18n.t('model_store.syncing')}</p>
            </div>
        );
    }

    return (
        <div className="max-w-6xl mx-auto p-8 space-y-12 h-full overflow-y-auto custom-scrollbar">
            {/* Header section */}
            <div className="flex items-start justify-between">
                <div className="space-y-2">
                    <div className="flex items-center gap-3">
                        <div className="p-3 bg-blue-600/10 rounded-2xl text-blue-500 border border-blue-500/20">
                            <Store size={32} />
                        </div>
                        <h1 className="text-4xl font-black text-zinc-100 tracking-tight">
                            {i18n.t('model_store.title')}
                        </h1>
                    </div>
                    <p className="text-zinc-500 max-w-2xl leading-relaxed pl-1">
                        {i18n.t('model_store.description')}
                    </p>
                </div>

                <div className="flex items-center gap-4 bg-zinc-900/50 p-4 rounded-3xl border border-zinc-800 shadow-xl">
                    <div className="text-right space-y-0.5">
                        <p className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest">{i18n.t('model_store.active_bunkers', { count: nodes.length })}</p>
                        <p className="text-sm font-black text-emerald-500 uppercase">{i18n.t('model_store.swarm_connected')}</p>
                    </div>
                    <div className="w-12 h-12 rounded-2xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-500">
                        <Shield size={28} />
                    </div>
                </div>
            </div>

            {/* Model Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {catalog.map((model) => (
                    <div key={model.id} className="group relative bg-zinc-900 rounded-3xl border border-zinc-800 p-6 flex flex-col hover:border-blue-500/40 transition-all shadow-lg hover:shadow-blue-500/5 overflow-hidden">
                        <div className="absolute top-0 right-0 p-6 opacity-[0.03] group-hover:opacity-[0.07] transition-opacity pointer-events-none">
                            <Zap size={120} />
                        </div>

                        <div className="flex justify-between items-start mb-4">
                            <div className="space-y-1">
                                <h3 className="text-xl font-bold text-zinc-100">{model.name}</h3>
                                <div className="flex items-center gap-2">
                                    <span className="text-[10px] font-bold bg-blue-500/10 text-blue-500 border border-blue-500/20 px-2 py-0.5 rounded uppercase">{model.provider}</span>
                                    <span className="text-[10px] font-mono text-zinc-500">{model.size}</span>
                                </div>
                            </div>
                            <div className="text-right">
                                <p className="text-[10px] font-bold text-zinc-500 uppercase tracking-tighter">{i18n.t('model_store.vram_req')}</p>
                                <p className="text-sm font-black text-zinc-300 font-mono tracking-tighter">{model.vram}</p>
                            </div>
                        </div>

                        <p className="text-xs text-zinc-400 leading-relaxed flex-1 mb-6">
                            {model.description}
                        </p>

                        <div className="flex flex-wrap gap-2 mb-8">
                            {model.tags.map((tag: string) => (
                                <span key={tag} className="text-[9px] font-bold text-zinc-500 border border-zinc-800 px-2 py-1 rounded bg-zinc-950/50 uppercase tracking-widest">
                                    {tag}
                                </span>
                            ))}
                        </div>

                        <div className="space-y-3">
                            <h4 className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest px-1">{i18n.t('model_store.deploy_to_node')}</h4>
                            <div className="grid grid-cols-1 gap-2">
                                {nodes.map(node => {
                                    const nodeVram = parseInt(node.metadata?.vram || '0');
                                    const modelVram = parseInt(model.vram);
                                    const isCompatible = nodeVram >= modelVram || nodeVram === 0;

                                    return (
                                        <button
                                            key={node.id}
                                            disabled={isInstalling !== null || !isCompatible}
                                            onClick={() => handleInstall(model.id, node.id)}
                                            className={`w-full flex items-center justify-between p-3 rounded-2xl border transition-all ${
                                                isCompatible 
                                                ? 'bg-zinc-950 border-zinc-800 hover:border-blue-500/40 hover:bg-blue-500/5 group/btn'
                                                : 'bg-zinc-950/50 border-red-500/10 opacity-60 cursor-not-allowed'
                                            }`}
                                        >
                                            <div className="flex items-center gap-2">
                                                <HardDrive size={14} className={isCompatible ? 'text-zinc-500 group-hover/btn:text-blue-400' : 'text-red-500/40'} />
                                                <div className="text-left">
                                                    <p className={`text-[11px] font-bold ${isCompatible ? 'text-zinc-300' : 'text-zinc-500'}`}>{node.name}</p>
                                                    <p className="text-[9px] text-zinc-600 font-mono">{node.address}</p>
                                                </div>
                                            </div>
                                            {isInstalling === model.id ? (
                                                <Activity size={14} className="animate-spin text-blue-500" />
                                            ) : isCompatible ? (
                                                <Download size={14} className="text-zinc-700 group-hover/btn:text-blue-500" />
                                            ) : (
                                                <Tooltip content={`Requires ${model.vram} VRAM. Node has ${node.metadata?.vram || 'Unknown'}.`} position="top">
                                                    <Info size={14} className="text-red-900" />
                                                </Tooltip>
                                            )}
                                        </button>
                                    );
                                })}
                            </div>
                        </div>
                    </div>
                ))}
            </div>
            
            {/* Hardware Profile Banner */}
            <div className="bg-gradient-to-r from-blue-900/20 to-emerald-900/10 border border-blue-500/20 rounded-3xl p-8 flex items-center justify-between relative overflow-hidden group">
                <div className="absolute top-0 right-0 w-64 h-64 bg-emerald-500/5 blur-[120px] rounded-full pointer-events-none" />
                <div className="space-y-2 z-10">
                    <h2 className="text-xl font-black text-zinc-100 uppercase tracking-tight flex items-center gap-2">
                        <Cpu size={24} className="text-emerald-500" />
                        {i18n.t('model_store.hardware_profile_title')}
                    </h2>
                    <p className="text-zinc-500 text-xs max-w-xl">
                        {i18n.t('model_store.hardware_profile_desc')}
                    </p>
                </div>
                <div className="z-10">
                    <button className="flex items-center gap-2 px-6 py-3 bg-zinc-950 border border-zinc-800 rounded-2xl text-xs font-bold text-zinc-300 hover:border-emerald-500/40 transition-all uppercase tracking-[0.2em]">
                        {i18n.t('model_store.btn_auto_optimize')}
                        <ChevronRight size={16} />
                    </button>
                </div>
            </div>
        </div>
    );
}
