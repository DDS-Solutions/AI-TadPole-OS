import { describe, it, expect } from 'vitest';
import { get_department_icon, get_agent_status_styles, get_theme_colors } from './agent_uiutils';
import { Shield, Cpu, Zap, Globe, ShieldCheck, Users, Target, Activity } from 'lucide-react';

describe('agentUIUtils', () => {
    describe('get_department_icon', () => {
        it('returns correct icons for known departments', () => {
            expect(get_department_icon('Executive')).toBe(Shield);
            expect(get_department_icon('Engineering')).toBe(Cpu);
            expect(get_department_icon('Operations')).toBe(Zap);
            expect(get_department_icon('Product')).toBe(Globe);
            expect(get_department_icon('Quality Assurance')).toBe(ShieldCheck);
            expect(get_department_icon('QA')).toBe(ShieldCheck);
            expect(get_department_icon('Design')).toBe(Target);
            expect(get_department_icon('Research')).toBe(Activity);
            expect(get_department_icon('Support')).toBe(Users);
        });

        it('returns a fallback icon for unknown departments', () => {
            expect(get_department_icon('Unknown')).toBe(Users);
            expect(get_department_icon('')).toBe(Users);
        });
    });

    describe('get_agent_status_styles', () => {
        it('returns emerald styles for active statuses', () => {
            const result = get_agent_status_styles('working');
            expect(result.text).toBe('text-emerald-400');
            expect(result.hex).toBe('#10b981');

            const result2 = get_agent_status_styles('coding');
            expect(result2.text).toBe('text-emerald-400');
        });

        it('returns amber styles for thinking status', () => {
            const result = get_agent_status_styles('thinking');
            expect(result.text).toBe('text-amber-400');
            expect(result.hex).toBe('#f59e0b');
        });

        it('returns blue styles for speaking status', () => {
            const result = get_agent_status_styles('speaking');
            expect(result.text).toBe('text-blue-400');
            expect(result.hex).toBe('#3b82f6');
        });

        it('returns zinc styles for paused and offline statuses', () => {
            const paused = get_agent_status_styles('paused');
            expect(paused.text).toBe('text-zinc-500');

            const offline = get_agent_status_styles('offline');
            expect(offline.text).toBe('text-red-500');
        });

        it('returns fallback styles for unknown or idle statuses', () => {
            const idle = get_agent_status_styles('idle');
            expect(idle.text).toBe('text-zinc-400');
            expect(idle.hex).toBe('#a1a1aa');

            const unknown = get_agent_status_styles('sleeping');
            expect(unknown.text).toBe('text-zinc-400');
        });
    });

    describe('get_theme_colors', () => {
        it('returns correct colors for known themes', () => {
            const cyan = get_theme_colors('cyan');
            expect(cyan.text).toBe('text-cyan-400');
            
            const zinc = get_theme_colors('zinc');
            expect(zinc.text).toBe('text-zinc-400');
            
            const amber = get_theme_colors('amber');
            expect(amber.text).toBe('text-amber-400');
        });

        it('returns fallback blue colors for unknown themes', () => {
            const fallback = get_theme_colors('unknown');
            expect(fallback.text).toBe('text-blue-400');
            
            const undefinedFallback = get_theme_colors(undefined);
            expect(undefinedFallback.text).toBe('text-blue-400');
        });
    });
});

