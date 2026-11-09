/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Assist Note
 * - Review changes preserve origin security and propagate confirmed operation outcomes.
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / General / AgentConfigPanel
 * - **Primary Entrypoints**: `AgentConfigPanel`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { useState, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { use_role_store } from '../stores/role_store';
import type { Role_State } from '../stores/role_store';
import type {
  Agent,
  Agent_Voice_Engine,
  Agent_Stt_Engine,
} from '../types';
import { i18n } from '../i18n';

// Decomposed Components
import {
  AgentConfigHeader,
  CognitionSection,
  VoiceSection,
  GovernanceSection,
  MemorySection,
  DirectMessageConsole,
  useAgentConfig,
  useAgentMemory,
  AgentConfigTabsBar,
  AgentConfigFooter,
} from './agent-config';
import { Portal_Window } from './ui/Portal_Window';
import { ExternalLink } from 'lucide-react';

interface AgentConfigPanelProps {
  agent: Agent | undefined;
  onClose: () => void;
  onUpdate: (id: string, updates: Partial<Agent>) => void | boolean | Promise<void | boolean>;
  onDelete?: (id: string) => void;
  isNew?: boolean;
  isDetachedMode?: boolean;
}

const formatAgentId = (id?: string): string => {
  if (!id || id === 'new') return 'NEW';
  return id.length >= 8 ? id.substring(0, 8).toUpperCase() : id.toUpperCase();
};

/**
 * AgentConfigPanel
 * Main orchestration component for agent configuration.
 * Manages tab state, detached window logic, and capability synchronization.
 */
export default function AgentConfigPanel({
  agent,
  onClose,
  onUpdate,
  onDelete,
  isNew = false,
  isDetachedMode = false,
}: AgentConfigPanelProps) {
  const {
    state,
    dispatch,
    handleRoleChange,
    handleProviderChange,
    handleSave,
    handlePause,
    handleResume,
    handleSendMessage,
    handlePromote,
    handleUpdateIdentity,
    handleUpdateThemeColor,
    handleSetTab,
    handleUpdateSlotField,
    handleToggleSkill,
    handleAddConnector,
    handleRemoveConnector,
    handleUpdateGovernance,
    handleUpdateVoice,
  } = useAgentConfig(agent, onUpdate, onClose);

  const {
    identity,
    slots,
    voice,
    ui,
    governance,
    main_tab: mainTab,
    active_tab: activeTab,
  } = state;

  const roles = use_role_store((s: Role_State) => s.roles);
  const [isDetached, setIsDetached] = useState(false);
  const agentId = agent?.id;

  const {
    memories,
    isLoadingMemories,
    memoryInput,
    setMemoryInput,
    loadMemories,
    handleSaveMemory,
    handleDeleteMemory,
  } = useAgentMemory(agentId, mainTab);

  const voiceProp = useMemo(
    () => ({
      voice_id: voice.voice_id as string,
      voice_engine: voice.voice_engine as Agent_Voice_Engine,
    }),
    [voice.voice_id, voice.voice_engine],
  );

  const safeThemeColor = useMemo(() => {
    return ui.theme_color && /^#[0-9A-F]{6}$/i.test(ui.theme_color) ? ui.theme_color : '#10b981';
  }, [ui.theme_color]);

  if (!agent && !ui.saving && !isNew) return null;

  const panelContent = (
    <div className="flex-1 flex flex-col min-h-0 bg-[color:var(--color-background)]/40 backdrop-blur-xl">
      <AgentConfigHeader
        name={identity.name}
        role={identity.role}
        department={identity.department}
        themeColor={safeThemeColor}
        isNew={isNew || !agent?.id}
        agentId={agent?.id}
        availableRoles={Object.keys(roles)}
        onClose={onClose}
        onDetach={() => setIsDetached(true)}
        onDelete={
          agent?.id && onDelete
            ? () => {
                onDelete(agent.id);
                onClose();
              }
            : undefined
        }
        isDetached={isDetached}
        onUpdateIdentity={handleUpdateIdentity}
        onUpdateThemeColor={handleUpdateThemeColor}
        onRoleChange={handleRoleChange}
      />

      <AgentConfigTabsBar
        mainTab={mainTab as 'cognition' | 'memory' | 'governance'}
        onTabChange={(tab) => dispatch({ type: 'SET_MAIN_TAB', payload: tab })}
      />

      <div className="flex-1 overflow-hidden flex flex-col relative min-h-0">
        <div className="flex-1 overflow-y-auto custom-scrollbar min-h-0">
          {mainTab === 'cognition' && (
            <CognitionSection
              activeTab={activeTab}
              slots={slots}
              agentStatus={agent?.status || 'idle'}
              themeColor={safeThemeColor}
              activeModelSlot={agent?.active_model_slot}
              onSetTab={handleSetTab}
              onUpdateSlotField={handleUpdateSlotField}
              onToggleSkill={handleToggleSkill}
              onProviderChange={handleProviderChange}
              onPause={handlePause}
              onResume={handleResume}
            />
          )}
          {mainTab === 'memory' && (
            <MemorySection
              memories={memories}
              connectorConfigs={state.connector_configs || []}
              isLoading={isLoadingMemories}
              memoryInput={memoryInput}
              themeColor={safeThemeColor}
              onMemoryInputChange={setMemoryInput}
              onSaveMemory={handleSaveMemory}
              onDeleteMemory={handleDeleteMemory}
              onRefresh={loadMemories}
              onAddConnector={handleAddConnector}
              onRemoveConnector={handleRemoveConnector}
            />
          )}
          {mainTab === 'governance' && (
            <GovernanceSection
              key={agent?.id || 'new'}
              budget_usd={governance.budget_usd}
              requires_oversight={governance.requires_oversight}
              shadows_human_id={governance.shadows_human_id}
              economic_zone={governance.economic_zone || 'DEV'}
              daily_spend_limit={governance.daily_spend_limit || 0}
              daily_spent_accumulated={agent?.daily_spent_accumulated || 0}
              balance={agent?.balance || 0}
              inventory={agent?.inventory || []}
              cost_usd={agent?.cost_usd || 0}
              theme_color={safeThemeColor}
              onUpdateGovernance={handleUpdateGovernance}
            />
          )}

          <div className="px-6 py-4 space-y-6 bg-[color:var(--color-surface)]/20 border-t border-[color:var(--color-surface)] shrink-0">
            <VoiceSection
              voice={voiceProp}
              stt_engine={(voice.stt_engine as Agent_Stt_Engine) || 'groq'}
              theme_color={safeThemeColor}
              on_update_voice={handleUpdateVoice}
            />
          </div>
        </div>

        <DirectMessageConsole
          value={ui.direct_message}
          onUpdateValue={(val) => dispatch({ type: 'SET_UI', field: 'direct_message', value: val })}
          onSend={handleSendMessage}
          agentName={identity.name}
          themeColor={safeThemeColor}
        />
      </div>

      <AgentConfigFooter
        showPromote={ui.show_promote}
        newRoleName={ui.new_role_name}
        isSaving={ui.saving}
        isNew={Boolean(isNew)}
        safeThemeColor={safeThemeColor}
        onTogglePromote={() =>
          dispatch({ type: 'SET_UI', field: 'show_promote', value: !ui.show_promote })
        }
        onNewRoleNameChange={(val) =>
          dispatch({ type: 'SET_UI', field: 'new_role_name', value: val })
        }
        onPromote={handlePromote}
        onSave={handleSave}
      />
    </div>
  );

  if (isDetached) {
    return (
      <AnimatePresence>
        <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 pointer-events-none">
          <Portal_Window
            id={`agent-config-${agent?.id || 'new'}`}
            title={`${identity.name || i18n.t('agent_config.new_agent')} (${formatAgentId(agent?.id)})`}
            url={`/detached-view?type=agent-config&id=${encodeURIComponent(agent?.id || 'new')}`}
            on_close={() => setIsDetached(false)}
          >
            {panelContent}
          </Portal_Window>

          <motion.div
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.9 }}
            className="relative w-full max-w-2xl aspect-video bg-[color:color-mix(in_srgb,var(--color-background)_80%,transparent)] backdrop-blur-2xl border border-[color:var(--color-surface)] rounded-[2.5rem] shadow-2xl flex flex-col items-center justify-center space-y-6 pointer-events-auto overflow-hidden"
          >
            <div className="absolute inset-0 neural-grid opacity-[0.05] pointer-events-none" />
            <div className="relative">
              <ExternalLink size={64} className="text-zinc-800 animate-pulse" />
              <div className="absolute inset-0 bg-green-500/10 blur-2xl rounded-full" />
            </div>
            <div className="text-center space-y-2 px-6">
              <h3 className="text-xl font-bold tracking-tight text-zinc-200">
                {i18n.t('layout.sector_detached')}
              </h3>
              <p className="text-sm text-zinc-500 font-mono uppercase tracking-[0.2em]">
                {i18n.t('layout.link_established')} :: ID_{formatAgentId(agent?.id)}
              </p>
            </div>
            <button
              onClick={() => setIsDetached(false)}
              className="px-6 py-2 bg-zinc-100 text-zinc-950 text-xs font-bold uppercase tracking-[0.2em] rounded-lg hover:bg-white transition-all shadow-lg active:scale-95 z-10"
            >
              {i18n.t('layout.recall_sector')}
            </button>
          </motion.div>
        </div>
      </AnimatePresence>
    );
  }

  if (isDetachedMode) {
    return (
      <div className="w-full h-full p-4 overflow-hidden flex flex-col">
        <div className="flex-1 min-h-0 bg-[color:var(--color-background)]/20 rounded-3xl border border-[color:var(--color-surface)] overflow-hidden shadow-2xl backdrop-blur-3xl relative">
          <div className="absolute inset-0 neural-grid opacity-[0.05] pointer-events-none" />
          {panelContent}
        </div>
      </div>
    );
  }

  return (
    <AnimatePresence>
      <div className="fixed inset-0 z-[100] flex items-center justify-center p-2 sm:p-6 md:p-10 pointer-events-none">
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={onClose}
          onKeyDown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              onClose();
              e.preventDefault();
            }
          }}
          tabIndex={0}
          role="button"
          aria-label="Close panel"
          className="absolute inset-0 bg-zinc-950/60 backdrop-blur-md pointer-events-auto"
        />

        <motion.div
          initial={{ opacity: 0, scale: 0.95, y: 20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.95, y: 20 }}
          transition={{ type: 'spring', damping: 25, stiffness: 300 }}
          className="relative w-full max-w-3xl  max-h-[95vh] sm:max-h-[90vh] bg-[color:var(--color-background)]/90 backdrop-blur-2xl rounded-[2.5rem] border border-white/5 shadow-[0_40px_100px_-20px_rgba(0,0,0,0.8)] flex flex-col overflow-hidden pointer-events-auto"
        >
          <div className="absolute inset-0 neural-grid opacity-[0.03] pointer-events-none" />
          <div className="flex-1 flex flex-col min-h-0">{panelContent}</div>
        </motion.div>
      </div>
    </AnimatePresence>
  );
}
