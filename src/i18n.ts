/**
 * @docs ARCHITECTURE:Infrastructure
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / i18n
 * - **Primary Entrypoints**: `i18n`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import en from './locales/en/index';

type LocaleData = typeof en;

interface TOptions {
  returnObjects?: boolean;
}

/**
 * I18n
 * Core internationalization class for the Tadpole OS ecosystem.
 */
class I18n {
  private data: LocaleData = en;

  /**
   * Namespace Routing Map
   * Maps legacy top-level keys to new modular namespaces.
   */
  private namespaceMap: Record<string, string> = {
    // Agent Domain
    'agent_card': 'agent',
    'agent_config': 'agent',
    'agent_manager': 'agent',
    'agent_role_select': 'agent',
    'agent_metrics': 'agent',
    'agent_details': 'agent',
    'memory_section': 'agent',
    
    // System & Infrastructure Domain
    'dashboard': 'system',
    'engine_dashboard': 'system',
    'metrics': 'system',
    'stats': 'system',
    'telemetry': 'system',
    'telemetry_graph': 'system',
    'settings': 'system',
    'benchmark': 'system',
    'layout': 'system',
    'hardware': 'system',
    'command': 'system',
    
    // Mission & Temporal Domain
    'missions': 'mission',
    'scheduled_jobs': 'mission',
    'workspaces': 'mission',
    'standups': 'mission',
    'transcript': 'mission',
    'voice': 'mission',
    
    // Navigation & Knowledge Domain
    'sidebar': 'nav',
    'docs': 'nav',
    
    // Intelligence & Skills Domain
    'provider': 'intelligence',
    'skills': 'intelligence',
    'model_store': 'intelligence',
    'template_store': 'intelligence',
    'model_manager': 'intelligence',
    
    // Interface & UX Domain
    'chat': 'interface',
    'swarm_visualizer': 'interface',
    
    // Oversight & Security Domain
    'oversight': 'security',
    
    // Observability Domain
    'trace': 'observability',
    'trace_stream': 'observability',
    'terminal': 'observability',
    'system_log': 'observability'
  };

  /**
   * t
   * Translates a key into a localized string or object.
   * Supports both legacy flat paths and new namespaced paths.
   */
  t(key: string, params: { returnObjects: true } & TOptions): Record<string, unknown>;
  t(key: string, params?: Record<string, string | number> | TOptions): string;
  t(key: string, params?: Record<string, string | number> | TOptions): string | Record<string, unknown> {
    let keys = key.split('.');
    
    // Check for Legacy Namespace Routing
    if (keys.length > 0 && this.namespaceMap[keys[0]]) {
      const newNamespace = this.namespaceMap[keys[0]];
      // If the legacy namespace is NOT the same as the new one, PREPEND it
      // This allows 'agent_config.title' to be looked up as 'agent.agent_config.title'
      if (newNamespace !== keys[0]) {
        keys = [newNamespace, ...keys];
      }
    }

    let result: unknown = this.data;
    
    for (const k of keys) {
      if (result && typeof result === 'object' && k in (result as Record<string, unknown>)) {
        result = (result as Record<string, unknown>)[k];
      } else {
        result = key;
        break;
      }
    }

    // If returnObjects is requested, return the raw result
    if (params && (params as TOptions).returnObjects) {
      return (result as Record<string, unknown>) || {};
    }
    
    let text = typeof result === 'string' ? result : key;
    
    if (params && !(params as TOptions).returnObjects) {
      Object.entries(params as Record<string, string | number>).forEach(([k, v]) => {
        const val = String(v);
        // Support both new {{param:key}} and legacy {{key}}
        text = text.replaceAll(`{{param:${k}}}`, val).replaceAll(`{{${k}}}`, val);
      });
    }
    
    return text;
  }
}

export const i18n = new I18n();
