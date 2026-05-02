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

import en from './locales/en.json';

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
   * t
   * Translates a key into a localized string or object.
   * 
   * @param key - The dot-notation path to the localization string (e.g. 'common.confirm')
   * @param params - Optional parameters for interpolation or {{returnObjects: true}} to get raw JSON chunks.
   * @returns The localized string or object.
   */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  t(key: string, params?: Record<string, string | number> | TOptions): any {
    const keys = key.split('.');
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
        text = text.replace(new RegExp(`{{${k}}}`, 'g'), String(v));
      });
    }
    
    return text;
  }
}

export const i18n = new I18n();

// Metadata: [i18n]

// Metadata: [i18n]
