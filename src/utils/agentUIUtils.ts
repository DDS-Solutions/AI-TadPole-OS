import { Shield, Cpu, Zap, Globe, ShieldCheck, Users, Target, Activity } from 'lucide-react';
import type { AgentStatus, Department } from '../types';

/**
 * Maps a department to its corresponding Lucide icon.
 */
export const getDepartmentIcon = (dept: Department | string) => {
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

/**
 * Maps an agent status to its core theme color (Tailwind-compatible strings and Hex).
 */
export const getAgentStatusStyles = (status: AgentStatus | string) => {
    switch (status) {
        case 'working':
        case 'active':
        case 'coding':
            return {
                text: 'text-emerald-400',
                border: 'border-emerald-500/50',
                bg: 'bg-emerald-500/10',
                glow: 'shadow-emerald-500/20',
                hex: '#10b981'
            };
        case 'thinking':
            return {
                text: 'text-amber-400',
                border: 'border-amber-500/50',
                bg: 'bg-amber-500/10',
                glow: 'shadow-amber-500/20',
                hex: '#f59e0b'
            };
        case 'speaking':
            return {
                text: 'text-blue-400',
                border: 'border-blue-500/50',
                bg: 'bg-blue-500/10',
                glow: 'shadow-blue-500/20',
                hex: '#3b82f6'
            };
        case 'paused':
            return {
                text: 'text-zinc-500',
                border: 'border-zinc-700/50',
                bg: 'bg-zinc-800/20',
                glow: 'none',
                hex: '#71717a'
            };
        case 'suspended':
            return {
                text: 'text-zinc-600',
                border: 'border-red-900/20',
                bg: 'bg-zinc-950/60',
                glow: 'none',
                hex: '#52525b'
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
                border: 'border-emerald-500/10',
                bg: 'bg-zinc-900/40',
                glow: 'shadow-emerald-500/5',
                hex: '#a1a1aa' // Zinc 400 - "Idle/Ready" state
            };
    }
};

/**
 * Standardized theme color extractor.
 */
export const getThemeColors = (theme: string = 'blue') => {
    switch (theme) {
        case 'cyan': return { text: 'text-cyan-400', border: 'border-cyan-500/50', bg: 'bg-cyan-500/5', glow: 'shadow-cyan-500/10', hex: '#22d3ee' };
        case 'zinc': return { text: 'text-zinc-400', border: 'border-zinc-500/50', bg: 'bg-zinc-500/5', glow: 'shadow-zinc-500/10', hex: '#a1a1aa' };
        case 'amber': return { text: 'text-amber-400', border: 'border-amber-500/50', bg: 'bg-amber-500/5', glow: 'shadow-amber-500/10', hex: '#fbbf24' };
        default: return { text: 'text-blue-400', border: 'border-blue-500/50', bg: 'bg-blue-500/5', glow: 'shadow-blue-500/10', hex: '#60a5fa' };
    }
};

/**
 * Maps emotional valence (-1.0 to 1.0) to color codes.
 * - Positive (> 0.2): Emerald/Green
 * - Negative (< -0.2): Red/Rose
 * - Neutral: Blue/Zinc
 */
export const getValenceColor = (valence: number = 0) => {
    if (valence > 0.3) return '#10b981'; // Emerald 500
    if (valence < -0.3) return '#ef4444'; // Red 500
    return '#3b82f6'; // Blue 500
};
