/**
 * @file workspace_store.ts (Legacy Bridge)
 * @description Bridges legacy components to the centralized workspace store.
 */
import { use_workspace_store as root_store } from '../../../src/stores/workspace_store';

export const use_workspace_store = root_store;
export const useWorkspaceStore = root_store; // Alias if needed
