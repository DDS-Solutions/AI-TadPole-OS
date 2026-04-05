/**
 * @docs ARCHITECTURE:Infrastructure
 * 
 * ### AI Assist Note
 * **Root/Core**: Manages the i18n. 
 * Part of the Tadpole-OS core layer.
 */

import en from './locales/en.json';

type LocaleData = typeof en;

class I18n {
  private data: LocaleData = en;

  t(key: string, params?: Record<string, string | number>): string {
    const keys = key.split('.');
    let result: unknown = this.data;
    
    for (const k of keys) {
      if (result && typeof result === 'object' && k in (result as Record<string, unknown>)) {
        result = (result as Record<string, unknown>)[k];
      } else {
        result = key;
        break;
      }
    }
    
    let text = typeof result === 'string' ? result : key;
    
    if (params) {
      Object.entries(params).forEach(([k, v]) => {
        text = text.replace(new RegExp(`{{${k}}}`, 'g'), String(v));
      });
    }
    
    return text;
  }
}

export const i18n = new I18n();

