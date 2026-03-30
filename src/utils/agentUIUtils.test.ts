import { describe, it, expect } from 'vitest';
import { getDepartmentIcon, getAgentStatusStyles, getThemeColors } from './agentUIUtils';
import { Shield, Cpu, Zap, Globe, ShieldCheck, Users, Target, Activity } from 'lucide-react';

describe('agentUIUtils', () => {
    describe('getDepartmentIcon', () => {
        it('returns correct icons for known departments', () => {
            expect(getDepartmentIcon('Executive')).toBe(Shield);
            expect(getDepartmentIcon('Engineering')).toBe(Cpu);
            expect(getDepartmentIcon('Operations')).toBe(Zap);
            expect(getDepartmentIcon('Product')).toBe(Globe);
            expect(getDepartmentIcon('Quality Assurance')).toBe(ShieldCheck);
            expect(getDepartmentIcon('QA')).toBe(ShieldCheck);
            expect(getDepartmentIcon('Design')).toBe(Target);
            expect(getDepartmentIcon('Research')).toBe(Activity);
            expect(getDepartmentIcon('Support')).toBe(Users);
        });

        it('returns a fallback icon for unknown departments', () => {
            expect(getDepartmentIcon('Unknown')).toBe(Users);
            expect(getDepartmentIcon('')).toBe(Users);
        });
    });

    describe('getAgentStatusStyles', () => {
        it('returns emerald styles for active statuses', () => {
            const result = getAgentStatusStyles('working');
            expect(result.text).toBe('text-emerald-400');
            expect(result.hex).toBe('#10b981');

            const result2 = getAgentStatusStyles('coding');
            expect(result2.text).toBe('text-emerald-400');
        });

        it('returns amber styles for thinking status', () => {
            const result = getAgentStatusStyles('thinking');
            expect(result.text).toBe('text-amber-400');
            expect(result.hex).toBe('#f59e0b');
        });

        it('returns blue styles for speaking status', () => {
            const result = getAgentStatusStyles('speaking');
            expect(result.text).toBe('text-blue-400');
            expect(result.hex).toBe('#3b82f6');
        });

        it('returns zinc styles for paused and offline statuses', () => {
            const paused = getAgentStatusStyles('paused');
            expect(paused.text).toBe('text-zinc-500');

            const offline = getAgentStatusStyles('offline');
            expect(offline.text).toBe('text-red-500');
        });

        it('returns fallback styles for unknown or idle statuses', () => {
            const idle = getAgentStatusStyles('idle');
            expect(idle.text).toBe('text-zinc-400');
            expect(idle.hex).toBe('#a1a1aa');

            const unknown = getAgentStatusStyles('sleeping');
            expect(unknown.text).toBe('text-zinc-400');
        });
    });

    describe('getThemeColors', () => {
        it('returns correct colors for known themes', () => {
            const cyan = getThemeColors('cyan');
            expect(cyan.text).toBe('text-cyan-400');
            
            const zinc = getThemeColors('zinc');
            expect(zinc.text).toBe('text-zinc-400');
            
            const amber = getThemeColors('amber');
            expect(amber.text).toBe('text-amber-400');
        });

        it('returns fallback blue colors for unknown themes', () => {
            const fallback = getThemeColors('unknown');
            expect(fallback.text).toBe('text-blue-400');
            
            const undefinedFallback = getThemeColors(undefined);
            expect(undefinedFallback.text).toBe('text-blue-400');
        });
    });
});
