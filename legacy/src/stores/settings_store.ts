/**
 * @file settings_store.ts (Legacy Bridge)
 * @description Bridges legacy components to the centralized settings store.
 */
import { use_settings_store as root_store } from '../../../src/stores/settings_store';

export const use_settings_store = root_store;
export const useSettingsStore = root_store; // Alias if needed
