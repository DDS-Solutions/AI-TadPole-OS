/**
 * @file skill_store.ts (Legacy Bridge)
 * @description Bridges legacy components to the centralized skill store.
 */
import { use_skill_store as root_store } from '../../../src/stores/skill_store';
export * from '../../../src/stores/skill_store'; // Export types

export const use_skill_store = root_store;
