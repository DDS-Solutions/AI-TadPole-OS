/**
 * @docs ARCHITECTURE:Logic
 * 
 * ### AI Assist Note
 * **Recruitment Velocity Hook**: Encapsulates rolling 24-hour agent creation velocity calculation 
 * and its 60-second synchronization ticker into an isolated, self-contained hook.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Stale recruitment velocity if interval unmount fails, or invalid date parsing.
 * - **Telemetry Link**: Search for `[use_recruitment_velocity]` in component logs.
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

// Metadata: [use_recruitment_velocity]
