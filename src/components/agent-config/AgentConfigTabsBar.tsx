/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / AgentConfig / TabsBar
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { i18n } from '../../i18n';

export type MainTabType = 'cognition' | 'memory' | 'governance';

interface AgentConfigTabsBarProps {
  mainTab: MainTabType;
  onTabChange: (tab: MainTabType) => void;
}

export function AgentConfigTabsBar({ mainTab, onTabChange }: AgentConfigTabsBarProps) {
  const MAIN_TABS = [
    {
      id: 'cognition' as const,
      label: i18n.t('agent_config.tab_cognition'),
      activeColor: 'text-emerald-400',
      indicatorBg: 'bg-emerald-500',
    },
    {
      id: 'memory' as const,
      label: i18n.t('agent_config.tab_memory'),
      activeColor: 'text-green-400',
      indicatorBg: 'bg-green-500',
    },
    {
      id: 'governance' as const,
      label: i18n.t('agent_config.tab_governance'),
      activeColor: 'text-amber-400',
      indicatorBg: 'bg-amber-500',
    },
  ];

  return (
    <div className="flex border-b border-[color:var(--color-surface)] bg-[color:var(--color-surface)]/40 px-6 shrink-0 z-10">
      {MAIN_TABS.map((tab) => (
        <button
          key={tab.id}
          onClick={() => onTabChange(tab.id)}
          className={`px-4 py-3 text-[10px] font-bold uppercase tracking-[0.2em] transition-all relative ${
            mainTab === tab.id ? tab.activeColor : 'text-zinc-500 hover:text-zinc-300'
          }`}
        >
          {tab.label}
          {mainTab === tab.id && (
            <div className={`absolute bottom-0 left-0 w-full h-0.5 ${tab.indicatorBg}`} />
          )}
        </button>
      ))}
    </div>
  );
}

