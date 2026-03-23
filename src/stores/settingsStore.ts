import { create } from 'zustand';
import { persist } from 'zustand/middleware';

const SETTINGS_KEY = 'tadpole_settings';

export interface TadpoleSettings {
    TadpoleOSUrl: string;
    TadpoleOSApiKey: string;
    theme: string;
    density: string;
    defaultModel: string;
    defaultTemperature: number;
    autoApproveSafeSkills: boolean;
    maxAgents: number;
    maxClusters: number;
    maxSwarmDepth: number;
    maxTaskLength: number;
    defaultBudgetUsd: number;
    isSafeMode: boolean;
    privacyMode: boolean;
}

interface SettingsState {
    settings: TadpoleSettings;
    saveSettings: (newSettings: TadpoleSettings) => string | null;
    updateSetting: <K extends keyof TadpoleSettings>(key: K, value: TadpoleSettings[K]) => void;
}

const getBaseUrl = () => {
    try {
        return `${window.location.protocol}//${window.location.hostname}:8000`;
    } catch {
        return 'http://localhost:8000';
    }
};

/** Simple base64 obfuscation — not encryption, but prevents casual shoulder-surfing. */
const obfuscate = (value: string): string => value ? btoa(value) : '';
const deobfuscate = (value: string): string => {
    try { return value ? atob(value) : ''; }
    catch { return ''; }
};

/** Validates a URL string. Rejects non-http(s) protocols. */
export function isValidUrl(url: string): boolean {
    try {
        const parsed = new URL(url);
        return parsed.protocol === 'http:' || parsed.protocol === 'https:';
    } catch {
        return false;
    }
}

export const useSettingsStore = create<SettingsState>()(
    persist(
        (set) => ({
            settings: {
                TadpoleOSUrl: getBaseUrl(),
                TadpoleOSApiKey: 'tadpole-dev-token-2026',
                theme: 'zinc',
                density: 'compact',
                defaultModel: 'GPT-4o',
                defaultTemperature: 0.7,
                autoApproveSafeSkills: true,
                maxAgents: 50,
                maxClusters: 10,
                maxSwarmDepth: 5,
                maxTaskLength: 32768,
                defaultBudgetUsd: 1.0,
                isSafeMode: false,
                privacyMode: false,
            } as unknown as TadpoleSettings,

            saveSettings: (newSettings) => {
                if (!isValidUrl(newSettings.TadpoleOSUrl)) {
                    return 'Invalid URL. Must start with http:// or https://';
                }
                set({ settings: newSettings });
                return null;
            },

            updateSetting: (key, value) => set(state => ({
                settings: { ...state.settings, [key]: value }
            }))
        }),
        {
            name: SETTINGS_KEY,
            storage: {
                getItem: (name) => {
                    const str = localStorage.getItem(name);
                    if (!str) return null;
                    try {
                        const parsed = JSON.parse(str);
                        const defaults = {
                            TadpoleOSUrl: getBaseUrl(),
                            TadpoleOSApiKey: 'tadpole-dev-token-2026',
                            theme: 'zinc',
                            density: 'compact',
                            defaultModel: 'GPT-4o',
                            defaultTemperature: 0.7,
                            autoApproveSafeSkills: true,
                            maxAgents: 50,
                            maxClusters: 10,
                            maxSwarmDepth: 5,
                            maxTaskLength: 32768,
                            defaultBudgetUsd: 1.0,
                            isSafeMode: false,
                            privacyMode: false,
                        };
                        const settings = { ...defaults, ...parsed.state.settings };

                        // MIGRATION: Handle legacy OpenClaw naming
                        const legacyUrl = (parsed.state.settings as Record<string, unknown>).openClawUrl as string | undefined;
                        const legacyKey = (parsed.state.settings as Record<string, unknown>).openClawApiKey as string | undefined;

                        if (legacyUrl && (!settings.TadpoleOSUrl || settings.TadpoleOSUrl === defaults.TadpoleOSUrl)) {
                            console.debug(`[Settings] Migrating legacy URL: ${legacyUrl} -> TadpoleOSUrl`);
                            settings.TadpoleOSUrl = legacyUrl;
                        }
                        if (legacyKey && (!settings.TadpoleOSApiKey || settings.TadpoleOSApiKey === defaults.TadpoleOSApiKey)) {
                            console.debug(`[Settings] Migrating legacy API Key -> TadpoleOSApiKey`);
                            settings.TadpoleOSApiKey = deobfuscate(legacyKey);
                        }

                        if (settings.TadpoleOSApiKey) {
                            settings.TadpoleOSApiKey = deobfuscate(settings.TadpoleOSApiKey);
                        }

                        let storedUrl = (settings.TadpoleOSUrl || defaults.TadpoleOSUrl).trim();
                        const isLocalAlias = /^(http|ws)s?:\/\/(localhost|127\.0\.0\.1|0\.0\.0\.0)(:|$)/i.test(storedUrl);
                        const isRemoteAccess = window.location.hostname !== 'localhost' && window.location.hostname !== '10.0.0.1';

                        if (isLocalAlias && isRemoteAccess) {
                            console.debug(`[Settings] Auto-fixing local agent URL: ${storedUrl} -> ${window.location.protocol}//${window.location.hostname}:8000`);
                            storedUrl = `${window.location.protocol}//${window.location.hostname}:8000`;
                            settings.TadpoleOSUrl = storedUrl;
                        }

                        return {
                            ...parsed,
                            state: { ...parsed.state, settings }
                        };
                    } catch {
                        return null;
                    }
                },
                setItem: (name, value) => {
                    const settings = { ...value.state.settings };
                    if (settings.TadpoleOSApiKey) {
                        settings.TadpoleOSApiKey = obfuscate(settings.TadpoleOSApiKey);
                    }
                    localStorage.setItem(name, JSON.stringify({
                        ...value,
                        state: { ...value.state, settings }
                    }));
                },
                removeItem: (name) => localStorage.removeItem(name),
            }
        }
    )
);

// Backward compatibility helpers for non-reactive code (like TadpoleOSService)
export const getSettings = () => useSettingsStore.getState().settings;
export const saveSettings = (s: TadpoleSettings) => useSettingsStore.getState().saveSettings(s);

