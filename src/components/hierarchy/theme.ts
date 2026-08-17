/**
 * @docs ARCHITECTURE:UI
 * 
 * ### AI Assist Note
 * Theme styles and color resolution helpers for the Org Chart hierarchy view.
 * Maps cluster themes to background glows, pulse animations, and header tints.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Unmapped theme key fallback failure.
 * - **Telemetry Link**: Search `[HierarchyTheme]` in UI logs.
 */

export type ThemeKey = 'cyan' | 'zinc' | 'amber';

export const THEME_CLASSES: Record<ThemeKey, { bg: string; pulse: string; heading: string }> = {
    cyan: { bg: 'bg-cyan-500/20', pulse: 'vertical-pulse text-cyan-500', heading: 'text-cyan-400' },
    zinc: { bg: 'bg-zinc-500/20', pulse: 'vertical-pulse text-zinc-500', heading: 'text-zinc-400' },
    amber: { bg: 'bg-amber-500/20', pulse: 'vertical-pulse text-amber-500', heading: 'text-amber-400' },
};

export const getThemeClasses = (theme: string) => {
    const key = (theme === 'cyan' || theme === 'amber' ? theme : 'zinc') as ThemeKey;
    return THEME_CLASSES[key];
};

// Metadata: [HierarchyTheme]
