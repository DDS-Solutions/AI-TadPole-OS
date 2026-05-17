/**
 * @docs ARCHITECTURE:Infrastructure
 * 
 * ### AI Assist Note
 * **Root/Core**: Lifecycle-managed internationalization engine for Tadpole-OS. 
 * Orchestrates synchronous string lookups and dynamic object retrieval for department registries and system units.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Key missing in `en.json` (returns the key string), circular object reference during `returnObjects: true`, or interpolation failure due to missing param keys.
 * - **Telemetry Link**: Search for `[I18n]` or `i18n.t` in browser logs.
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
    
    // System & Infrastructure Domain
    'engine_dashboard': 'system',
    'metrics': 'system',
    'stats': 'system',
    'telemetry': 'system',
    'telemetry_graph': 'system',
    'settings': 'system',
    'benchmark': 'system',
    'layout': 'system',
    'hardware': 'system',
    
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
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  t(key: string, params?: Record<string, string | number> | TOptions): any {
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

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    let result: any = this.data;
    
    for (const k of keys) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      if (result && typeof result === 'object' && k in (result as Record<string, any>)) {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        result = (result as Record<string, any>)[k];
      } else {
        result = key;
        break;
      }
    }

    // If returnObjects is requested, return the raw result
    if (params && (params as TOptions).returnObjects) {
      return result;
    }
    
    let text = typeof result === 'string' ? result : key;
    
    if (params && !(params as TOptions).returnObjects) {
      Object.entries(params as Record<string, string | number>).forEach(([k, v]) => {
        const val = String(v);
        // Support both new {{param:key}} and legacy {{key}}
        text = text.replace(new RegExp(`{{param:${k}}}`, 'g'), val);
        text = text.replace(new RegExp(`{{${k}}}`, 'g'), val);
      });
    }
    
    return text;
  }
}

export const i18n = new I18n();

// Metadata: [i18n]
