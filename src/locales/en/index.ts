/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[index]` in observability traces.
 */

import common from './common.json';
import system from './system.json';
import nav from './nav.json';
import agent from './agent.json';
import mission from './mission.json';
import security from './security.json';
import intelligence from './intelligence.json';
import observability from './observability.json';
import interface_labels from './interface.json';
import glossary from './glossary.json';

const en = {
  ...common,
  ...system,
  ...nav,
  ...agent,
  ...mission,
  ...security,
  ...intelligence,
  ...observability,
  ...interface_labels,
  ...glossary,
  common,
  system,
  nav,
  agent,
  mission,
  security,
  intelligence,
  observability,
  interface: interface_labels,
  glossary
} as const;

export default en;
export type TranslationKeys = typeof en;

// Metadata: [index]
