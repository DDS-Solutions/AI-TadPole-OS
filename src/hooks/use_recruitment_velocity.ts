/**
 * @docs ARCHITECTURE:Logic
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / use_recruitment_velocity
 * - **Primary Entrypoints**: `useRecruitmentVelocity`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { useState, useEffect, useMemo } from 'react';
import type { Agent } from '../types';

export function useRecruitmentVelocity(agents_list: Agent[]) {
  const [now, set_now] = useState(() => Date.now());

  useEffect(() => {
    const interval = setInterval(() => {
      set_now(Date.now());
    }, 60000); // Synchronize window every 60 seconds
    return () => clearInterval(interval);
  }, []);

  const recruit_velocity = useMemo(() => {
    const twenty_four_hours_ago = now - 24 * 60 * 60 * 1000;
    const safe_agents = Array.isArray(agents_list) ? agents_list : [];
    return safe_agents.filter((a: Agent) => {
      if (!a.created_at) return false;
      const created_time = new Date(a.created_at).getTime();
      return !isNaN(created_time) && created_time > twenty_four_hours_ago;
    }).length;
  }, [agents_list, now]);

  return recruit_velocity;
}
