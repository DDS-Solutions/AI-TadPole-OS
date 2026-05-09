/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:Infrastructure**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[agent_uiutils]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:Infrastructure
 * @docs OPERATIONS_MANUAL:Navigation
 * 
 * ### AI Assist Note
 * **Infrastructure Utilities**: Global mapping functions for agent-related UI states. 
 */

console.debug("[AgentUIUtils] Module loaded");

import { Shield, Cpu, Zap, Globe, ShieldCheck, Users, Target, Activity } from 'lucide-react';
import type { Agent_Status, Department } from '../types';

export const get_department_icon = (dept: Department | string) => {
    switch (dept) {
        case 'Executive': return Shield;
        case 'Engineering': return Cpu;
        case 'Operations': return Zap;
        case 'Product': return Globe;
        case 'Quality Assurance':
        case 'QA':
            return ShieldCheck;
        case 'Design': return Target;
        case 'Research': return Activity;
        case 'Support': return Users;
        default: return Users;
    }
};

export const get_agent_status_styles = (status: Agent_Status | string) => {
    switch (status) {
        case 'working':
        case 'active':
        case 'coding':
            return {
                text: 'text-emerald-400',
                border: 'border-emerald-500/30',
                bg: 'bg-emerald-500/10',
                glow: 'shadow-emerald-500/20',
                hex: '#10b981'
            };
        case 'thinking':
            return {
                text: 'text-amber-400',
                border: 'border-amber-500/30',
                bg: 'bg-amber-500/10',
                glow: 'shadow-amber-500/20',
                hex: '#f59e0b'
            };
        case 'speaking':
            return {
                text: 'text-blue-400',
                border: 'border-blue-500/30',
                bg: 'bg-blue-500/10',
                glow: 'shadow-blue-500/20',
                hex: '#3b82f6'
            };
        case 'paused':
            return {
                text: 'text-zinc-500',
                border: 'border-zinc-800',
                bg: 'bg-zinc-900/40',
                glow: 'none',
                hex: '#71717a'
            };
        case 'offline':
            return {
                text: 'text-red-500',
                border: 'border-red-900/50',
                bg: 'bg-red-900/10',
                glow: 'none',
                hex: '#ef4444'
            };
        case 'idle':
        default:
            return {
                text: 'text-zinc-400',
                border: 'border-zinc-800',
                bg: 'bg-zinc-900/40',
                glow: 'none',
                hex: '#a1a1aa'
            };
    }
};

export const get_theme_colors = (theme: string = 'emerald') => {
    switch (theme) {
        case 'cyan': return { text: 'text-cyan-400', border: 'border-cyan-500/30', bg: 'bg-cyan-500/5', glow: 'shadow-cyan-500/10', hex: '#22d3ee' };
        case 'zinc': return { text: 'text-zinc-400', border: 'border-zinc-800', bg: 'bg-zinc-900/40', glow: 'none', hex: '#a1a1aa' };
        case 'amber': return { text: 'text-amber-400', border: 'border-amber-500/30', bg: 'bg-amber-500/5', glow: 'shadow-amber-500/10', hex: '#fbbf24' };
        case 'blue': return { text: 'text-blue-400', border: 'border-blue-500/30', bg: 'bg-blue-500/5', glow: 'shadow-blue-500/10', hex: '#3b82f6' };
        default: return { text: 'text-emerald-400', border: 'border-emerald-500/30', bg: 'bg-emerald-500/5', glow: 'shadow-emerald-500/10', hex: '#10b981' };
    }
};

export const get_valence_color = (valence: number = 0) => {
    if (valence > 0.3) return '#10b981'; // Emerald 500
    if (valence < -0.3) return '#ef4444'; // Red 500
    return '#3b82f6'; // Blue 500
};

// Metadata: [agent_uiutils]
