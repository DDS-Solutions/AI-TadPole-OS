/**
 * @docs ARCHITECTURE:UI
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Pages / Customer_Catalog_Manager
 * - **Primary Entrypoints**: `Customer_Catalog_Manager`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[Customer_Catalog_Manager]`
 */

import React, { useState, useMemo, useCallback, useEffect } from 'react';
import { Cpu, Store, Shield, Lock } from 'lucide-react';
import { A2A_Card_Preview, type ModelProfile } from '../components/outward/A2A_Card_Preview';
import { Company_Info_Card_Manager } from '../components/outward/Company_Info_Card_Manager';
import { CatalogImportCard } from '../components/outward/CatalogImportCard';
import { CatalogSearchTester } from '../components/outward/CatalogSearchTester';
import { useCatalogIngestion } from '../components/outward/useCatalogIngestion';
import { Tooltip } from '../components/ui';
import type { CatalogItem } from '../utils/csv_parser';
import { DEFAULT_INFO_CARDS, compileInfoCardsToSkills, type InfoCard } from '../utils/agent_card_compiler';
import { api_request } from '../services/base_api_service';
import { use_settings_store } from '../stores/settings_store';

export const Customer_Catalog_Manager: React.FC = () => {
  const [businessName, setBusinessName] = useState('Tadpole SMB Solutions');
  const [modelProfile, setModelProfile] = useState<ModelProfile>('gemma4:e4b');
  const [infoCards, setInfoCards] = useState<InfoCard[]>([...DEFAULT_INFO_CARDS]);
  const [isLoaded, setIsLoaded] = useState(false);
  const [syncStatus, setSyncStatus] = useState<'idle' | 'syncing' | 'synced' | 'error'>('idle');
  const [syncError, setSyncError] = useState<string | null>(null);

  const activeSkills = compileInfoCardsToSkills(infoCards);
  const tadpole_os_url = use_settings_store((state) => state.settings.tadpole_os_url);

  const previewUrl = useMemo(() => {
    const raw = tadpole_os_url?.trim() || (typeof window !== 'undefined' ? window.location.origin : 'http://localhost:8000');
    const base = raw.replace(/\/+$/, '');
    return `${base}/a2a/v1/company-agent-card.json`;
  }, [tadpole_os_url]);

  // Initial fetch of published agent card to align local UI with engine state
  useEffect(() => {
    let mounted = true;
    const fetchExistingCard = async () => {
      try {
        const card = await api_request<{
          name?: string;
          model_profile?: ModelProfile;
          skills?: Array<{ id: string; name: string; description: string; tags: string[] }>;
        }>('/a2a/v1/company-agent-card.json', { method: 'GET' });

        if (mounted && card) {
          if (card.name) setBusinessName(card.name);
          if (card.model_profile) setModelProfile(card.model_profile);
          setIsLoaded(true);
        }
      } catch (err) {
        if (mounted) {
          console.warn('[Customer_Catalog_Manager] Failed to fetch published agent card on mount, using initial state:', err);
          setIsLoaded(true);
        }
      }
    };
    fetchExistingCard();
    return () => {
      mounted = false;
    };
  }, []);

  // Debounced authenticated sync of published agent-card metadata to the Rust backend.
  useEffect(() => {
    if (!isLoaded) return;

    const controller = new AbortController();

    const timeout = window.setTimeout(async () => {
      setSyncStatus('syncing');
      setSyncError(null);
      try {
        await api_request('/a2a/v1/profile', {
          method: 'PUT',
          body: JSON.stringify({
            business_name: businessName,
            model_profile: modelProfile,
            skills: compileInfoCardsToSkills(infoCards),
          }),
          signal: controller.signal,
        });
        setSyncStatus('synced');
      } catch (err) {
        if (controller.signal.aborted) return;
        const msg = err instanceof Error ? err.message : 'Failed to synchronize published card';
        console.warn('[Customer_Catalog_Manager] A2A Gateway auto-sync warning:', err);
        setSyncStatus('error');
        setSyncError(msg);
      }
    }, 300);

    return () => {
      window.clearTimeout(timeout);
      controller.abort();
    };
  }, [businessName, modelProfile, infoCards, isLoaded]);

  // Catalog Search Tester State
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<CatalogItem[]>([
    { id: 'item-1', title: 'Heavy Duty Claw Hammer 16oz', category: 'Tools', price: '$18.99' },
    { id: 'item-2', title: 'Cordless 20V Power Drill', category: 'Power Tools', price: '$89.99' },
    { id: 'item-3', title: 'Professional Safety Eyewear', category: 'Apparel', price: '$12.50' },
  ]);

  const handleItemsIngested = useCallback((newItems: CatalogItem[]) => {
    setSearchResults((prev) => [...newItems, ...prev]);
  }, []);

  const {
    importStatus,
    dismissStatus,
    isProcessing,
    activeIngestionType,
    csvInputRef,
    qbInputRef,
    pdfInputRef,
    handleCsvFileChange,
    handleQbFileChange,
    handlePdfFileChange,
    triggerCsvUpload,
    triggerQbUpload,
    triggerPdfUpload,
  } = useCatalogIngestion({ onItemsIngested: handleItemsIngested });

  const filteredItems = useMemo(() => {
    const query = searchQuery.trim().toLowerCase();
    if (!query) return searchResults;
    return searchResults.filter((item) =>
      item.title.toLowerCase().includes(query) ||
      item.category.toLowerCase().includes(query)
    );
  }, [searchQuery, searchResults]);

  return (
    <div data-testid="route-customer-catalog" className="p-6 max-w-7xl mx-auto space-y-8 font-sans">
      {/* Header Banner */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-zinc-800/80 pb-6">
        <div>
          <h1 className="text-2xl font-bold text-zinc-100 flex items-center gap-2.5 tracking-tight">
            <Store className="w-7 h-7 text-emerald-400 shrink-0" />
            Customer Catalog & A2A Gateway Manager
          </h1>
          <p className="text-sm text-zinc-400 mt-1">
            Configure public customer-facing agent identity, ingest business catalogs, and publish compliant A2A Agent Cards.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Tooltip content="Zero-Trust Silo Barrier: Customer agents are isolated from internal codebase symbol graphs and system files." position="bottom">
            <div className="flex items-center gap-1.5 px-3 py-1.5 bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 rounded-full text-xs font-mono cursor-help">
              <Shield className="w-4 h-4" />
              <span>Silo Isolation Active</span>
            </div>
          </Tooltip>

          <Tooltip content="Anti-Scraping Shield: Caps public A2A discovery & catalog queries at 60 requests per minute per IP." position="bottom">
            <div className="flex items-center gap-1.5 px-3 py-1.5 bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 rounded-full text-xs font-mono cursor-help">
              <Lock className="w-4 h-4" />
              <span>60 req/min IP Shield</span>
            </div>
          </Tooltip>
        </div>
      </div>

      {syncError && (
        <div className="bg-red-500/10 border border-red-500/20 px-4 py-3 rounded-xl flex items-center justify-between text-red-400 text-xs font-mono">
          <span>⚠️ Profile Sync Error: {syncError}. Preview may not reflect published state.</span>
          <button 
            onClick={() => setSyncError(null)}
            className="text-red-300 hover:text-red-100 underline text-[11px] ml-4 cursor-pointer"
          >
            Dismiss
          </button>
        </div>
      )}

      {syncStatus === 'syncing' && (
        <div className="bg-cyan-500/10 border border-cyan-500/20 px-4 py-2 rounded-xl flex items-center gap-2 text-cyan-400 text-xs font-mono">
          <div className="w-2 h-2 rounded-full bg-cyan-400 animate-pulse" />
          <span>Synchronizing card profile with engine...</span>
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Left Column: Config & Ingestion */}
        <div className="space-y-6">
          {/* Business & Model Runner Settings */}
          <div className="bg-zinc-900/60 backdrop-blur-xl border border-white/5 rounded-2xl p-6 space-y-4 shadow-2xl relative overflow-hidden">
            <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-cyan-500/30 to-transparent" />
            <h2 className="text-base font-semibold text-zinc-200 flex items-center gap-2 tracking-tight">
              <Cpu className="w-5 h-5 text-cyan-400" />
              Business & Model Execution Profile
            </h2>

            <div className="space-y-4 pt-1">
              <div>
                <label className="block text-xs font-semibold text-zinc-400 uppercase tracking-wider mb-1.5">
                  Business Name (Public Identity)
                </label>
                <input
                  type="text"
                  value={businessName}
                  onChange={(e) => setBusinessName(e.target.value)}
                  className="w-full px-3.5 py-2.5 bg-zinc-950/40 border border-zinc-800 rounded-xl text-sm text-zinc-100 focus:outline-none focus:border-emerald-500/50 focus:ring-1 focus:ring-emerald-500/30 transition-all"
                  placeholder="e.g. Acme Hardware & Supply"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-zinc-400 uppercase tracking-wider mb-1.5">
                  Outward Model Execution Profile
                </label>
                <Tooltip content="Binds public customer queries to local inference profiles. gemma4:e4b delivers sub-500ms speed while preserving VRAM." position="top">
                  <select
                    value={modelProfile}
                    onChange={(e) => setModelProfile(e.target.value as ModelProfile)}
                    className="w-full px-3.5 py-2.5 bg-zinc-950/40 border border-zinc-800 rounded-xl text-sm text-zinc-100 focus:outline-none focus:border-cyan-500/50 focus:ring-1 focus:ring-cyan-500/30 cursor-pointer font-mono transition-all"
                  >
                    <option value="gemma4:e4b">gemma4:e4b (Recommended - Low VRAM, Sub-500ms)</option>
                    <option value="gemma4:e8b">gemma4:e8b (Standard Business Reasoning)</option>
                    <option value="gemma4:full">gemma4:full (High Density Execution)</option>
                  </select>
                </Tooltip>
              </div>
            </div>
          </div>

          {/* 1-Click SMB Catalog Ingestion */}
          <CatalogImportCard
            isProcessing={isProcessing}
            activeType={activeIngestionType}
            importStatus={importStatus}
            onDismissStatus={dismissStatus}
            onCsvClick={triggerCsvUpload}
            onQbClick={triggerQbUpload}
            onPdfClick={triggerPdfUpload}
            csvInputRef={csvInputRef}
            qbInputRef={qbInputRef}
            pdfInputRef={pdfInputRef}
            onCsvFileChange={handleCsvFileChange}
            onQbFileChange={handleQbFileChange}
            onPdfFileChange={handlePdfFileChange}
          />

          {/* Live Catalog Search Query Tester */}
          <CatalogSearchTester
            searchQuery={searchQuery}
            onSearchChange={setSearchQuery}
            filteredItems={filteredItems}
          />

          {/* Business FAQ & Operating Info Knowledge Card Manager */}
          <Company_Info_Card_Manager cards={infoCards} onCardsChange={setInfoCards} />
        </div>

        {/* Right Column: Live A2A Agent Card Preview */}
        <div className="space-y-4">
          <h2 className="text-base font-semibold text-zinc-200 flex items-center gap-2 tracking-tight">
            Published A2A Company Agent Card (<code className="font-mono text-xs text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded border border-emerald-500/20">company-agent-card.json</code>)
          </h2>
          <A2A_Card_Preview 
            name={businessName || 'Unnamed Business'} 
            modelProfile={modelProfile} 
            skills={activeSkills} 
            url={previewUrl} 
          />
        </div>
      </div>
    </div>
  );
};

export default Customer_Catalog_Manager;
