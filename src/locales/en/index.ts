/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / index
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
