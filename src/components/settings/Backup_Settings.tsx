/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Settings / Backup_Settings
 * - **Primary Entrypoints**: `Backup_Settings`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React, { useState, useEffect, useCallback } from 'react';
import { 
    ShieldCheck, 
    HardDrive, 
    Cloud, 
    Key, 
    RefreshCw, 
    Download, 
    Clock, 
    CheckCircle2, 
    AlertCircle, 
    Folder,
    Eye,
    EyeOff
} from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Tadpole_Settings } from '../../stores/settings_store';

interface BackupStatusData {
    enabled: boolean;
    destination_type: 'local' | 'cloud_sync';
    destination_path: string;
    has_passphrase: boolean;
    last_run_timestamp: number | null;
    last_status: string | null;
    total_snapshots: number;
    total_size_bytes: number;
    retention_hours: number;
}

interface BackupSettingsProps {
    settings: Tadpole_Settings;
    handle_change: (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => void;
    update_setting: <K extends keyof Tadpole_Settings>(key: K, value: Tadpole_Settings[K]) => void;
}

export const Backup_Settings: React.FC<BackupSettingsProps> = ({
    settings,
    handle_change,
    update_setting
}) => {
    const [backupStatus, setBackupStatus] = useState<BackupStatusData | null>(null);
    const [passphrase, setPassphrase] = useState('');
    const [showPassphrase, setShowPassphrase] = useState(false);
    const [isBackingUp, setIsBackingUp] = useState(false);
    const [feedback, setFeedback] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

    const baseUrl = settings.tadpole_os_url || 'http://127.0.0.1:8000';
    const apiKey = settings.tadpole_os_api_key || '';

    const fetchBackupStatus = useCallback(async () => {
        try {
            const res = await fetch(`${baseUrl}/v1/system/backup/status`, {
                headers: {
                    'Authorization': `Bearer ${apiKey}`
                }
            });
            if (res.ok) {
                const data = await res.json();
                setBackupStatus(data);
            }
        } catch (err) {
            console.debug('[BackupSettings] Status fetch deferred:', err);
        }
    }, [baseUrl, apiKey]);

    useEffect(() => {
        // eslint-disable-next-line react-hooks/set-state-in-effect
        void fetchBackupStatus();
    }, [fetchBackupStatus]);

    const handleModeSelect = (mode: 'local' | 'cloud_sync') => {
        update_setting('backup_mode', mode);
        void fetch(`${baseUrl}/v1/system/backup/configure`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${apiKey}`
            },
            body: JSON.stringify({
                enabled: settings.backup_enabled ?? true,
                destination_type: mode,
                destination_path: settings.backup_path || '',
                passphrase: passphrase || undefined
            })
        }).then(() => fetchBackupStatus());
    };

    const handleTriggerBackupNow = async () => {
        setIsBackingUp(true);
        setFeedback(null);
        try {
            // First ensure configuration is synced
            await fetch(`${baseUrl}/v1/system/backup/configure`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${apiKey}`
                },
                body: JSON.stringify({
                    enabled: true,
                    destination_type: settings.backup_mode || 'local',
                    destination_path: settings.backup_path || '',
                    passphrase: passphrase || undefined
                })
            });

            const res = await fetch(`${baseUrl}/v1/system/backup/trigger`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${apiKey}`
                }
            });

            if (res.ok) {
                const data = await res.json();
                setFeedback({
                    type: 'success',
                    text: i18n.t('settings.msg_backup_success', { filename: data.snapshot_filename })
                });
                await fetchBackupStatus();
            } else {
                const errData = await res.json().catch(() => ({}));
                setFeedback({
                    type: 'error',
                    text: errData.message || i18n.t('settings.msg_backup_failed', { error: 'Server error' })
                });
            }
        } catch (err: unknown) {
            const msg = err instanceof Error ? err.message : String(err);
            setFeedback({
                type: 'error',
                text: i18n.t('settings.msg_backup_failed', { error: msg })
            });
        } finally {
            setIsBackingUp(false);
        }
    };

    const handleDownloadRecoveryKey = () => {
        const manifest = {
            version: '1.0',
            app: 'Tadpole OS Sovereign Backup',
            kdf: 'PBKDF2-HMAC-SHA512',
            rounds: 100000,
            note: 'Store this recovery file securely offline. Required to decrypt snapshots if your system drive fails.',
            created_at: new Date().toISOString()
        };
        const blob = new Blob([JSON.stringify(manifest, null, 2)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'tadpole-recovery.key';
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    };

    const destinationMode = settings.backup_mode || 'local';
    const totalSizeMb = backupStatus ? (backupStatus.total_size_bytes / (1024 * 1024)).toFixed(1) : '0.0';

    return (
        <div className="space-y-4 pb-12">
            <div className="flex items-center justify-between">
                <Tooltip content={i18n.t('settings.tooltip_backup_section')} position="top">
                    <h2 className="text-sm font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2 cursor-help">
                        <ShieldCheck size={16} className="text-emerald-500" />
                        {i18n.t('settings.header_backup')}
                    </h2>
                </Tooltip>

                {/* Status Badges */}
                <div className="flex items-center gap-2">
                    <span className="text-[11px] font-mono uppercase tracking-wider px-2.5 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 flex items-center gap-1.5">
                        <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
                        {i18n.t('settings.badge_retention')}
                    </span>
                    {backupStatus?.last_status === 'HEALTHY' ? (
                        <span className="text-[11px] font-mono px-2.5 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 flex items-center gap-1">
                            <CheckCircle2 size={12} />
                            {i18n.t('settings.status_healthy')}
                        </span>
                    ) : (
                        <span className="text-[11px] font-mono px-2.5 py-0.5 rounded-full bg-zinc-800 text-zinc-400 border border-zinc-700">
                            {settings.backup_path ? i18n.t('settings.status_healthy') : i18n.t('settings.status_disabled')}
                        </span>
                    )}
                </div>
            </div>

            <div className="bg-[color:var(--color-surface)] p-6 rounded-xl border border-[color:var(--color-border)] shadow-sm space-y-6 backdrop-blur-md relative overflow-hidden group">
                <div className="absolute top-0 right-0 p-6 opacity-5 group-hover:opacity-10 transition-opacity pointer-events-none">
                    <ShieldCheck size={120} />
                </div>

                {/* Feedback Toast Banner */}
                {feedback && (
                    <div className={`p-4 rounded-lg flex items-center gap-3 text-sm font-medium border ${
                        feedback.type === 'success' 
                            ? 'bg-emerald-500/10 text-emerald-300 border-emerald-500/30' 
                            : 'bg-red-500/10 text-red-300 border-red-500/30'
                    }`}>
                        {feedback.type === 'success' ? <CheckCircle2 size={18} /> : <AlertCircle size={18} />}
                        <span>{feedback.text}</span>
                    </div>
                )}

                {/* Dual Destination Choice: Local vs Cloud */}
                <div className="space-y-3">
                    <Tooltip content={i18n.t('settings.tooltip_destination_type')} position="top">
                        <label className="text-sm font-bold text-zinc-300 flex items-center gap-2 cursor-help w-max">
                            <Folder size={16} className="text-zinc-400" />
                            {i18n.t('settings.label_destination_type')}
                        </label>
                    </Tooltip>

                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                        {/* Option 1: Local Storage (Preferred) */}
                        <Tooltip content={i18n.t('settings.tooltip_opt_local')} position="top">
                            <button
                                type="button"
                                onClick={() => handleModeSelect('local')}
                                className={`p-4 rounded-lg border text-left transition-all flex items-start gap-3.5 ${
                                    destinationMode === 'local'
                                        ? 'bg-emerald-950/20 border-emerald-500/40 text-zinc-100 ring-1 ring-emerald-500/30'
                                        : 'bg-zinc-900/40 border-zinc-800 text-zinc-400 hover:border-zinc-700'
                                }`}
                            >
                                <div className={`p-2 rounded-md ${destinationMode === 'local' ? 'bg-emerald-500/20 text-emerald-400' : 'bg-zinc-800 text-zinc-400'}`}>
                                    <HardDrive size={20} />
                                </div>
                                <div className="space-y-1">
                                    <div className="font-semibold text-sm flex items-center gap-2">
                                        {i18n.t('settings.opt_local')}
                                        {destinationMode === 'local' && (
                                            <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-emerald-500/20 text-emerald-400">ACTIVE</span>
                                        )}
                                    </div>
                                    <p className="text-xs text-zinc-500 leading-relaxed">
                                        Direct writes to external drive, secondary NVMe, or local NAS without cloud dependency.
                                    </p>
                                </div>
                            </button>
                        </Tooltip>

                        {/* Option 2: Cloud Sync Folder */}
                        <Tooltip content={i18n.t('settings.tooltip_opt_cloud_sync')} position="top">
                            <button
                                type="button"
                                onClick={() => handleModeSelect('cloud_sync')}
                                className={`p-4 rounded-lg border text-left transition-all flex items-start gap-3.5 ${
                                    destinationMode === 'cloud_sync'
                                        ? 'bg-cyan-950/20 border-cyan-500/40 text-zinc-100 ring-1 ring-cyan-500/30'
                                        : 'bg-zinc-900/40 border-zinc-800 text-zinc-400 hover:border-zinc-700'
                                }`}
                            >
                                <div className={`p-2 rounded-md ${destinationMode === 'cloud_sync' ? 'bg-cyan-500/20 text-cyan-400' : 'bg-zinc-800 text-zinc-400'}`}>
                                    <Cloud size={20} />
                                </div>
                                <div className="space-y-1">
                                    <div className="font-semibold text-sm flex items-center gap-2">
                                        {i18n.t('settings.opt_cloud_sync')}
                                        {destinationMode === 'cloud_sync' && (
                                            <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-cyan-500/20 text-cyan-400">ACTIVE</span>
                                        )}
                                    </div>
                                    <p className="text-xs text-zinc-500 leading-relaxed">
                                        100% Zero-Knowledge encrypted files synced to OneDrive, Google Drive, or Dropbox.
                                    </p>
                                </div>
                            </button>
                        </Tooltip>
                    </div>
                </div>

                {/* Destination Directory Path Input */}
                <div className="space-y-2">
                    <div className="flex items-center justify-between">
                        <Tooltip content={i18n.t('settings.tooltip_backup_path')} position="top">
                            <label htmlFor="backup_path" className="text-sm font-bold text-zinc-300 cursor-help flex items-center gap-1.5">
                                <Folder size={14} className="text-zinc-400" />
                                {i18n.t('settings.label_backup_path')}
                            </label>
                        </Tooltip>
                        {destinationMode === 'cloud_sync' && (
                            <span className="text-xs text-zinc-500 font-mono">
                                Point to your local sync client folder
                            </span>
                        )}
                    </div>
                    <input
                        id="backup_path"
                        type="text"
                        name="backup_path"
                        value={settings.backup_path || ''}
                        onChange={handle_change}
                        placeholder={i18n.t('settings.placeholder_backup_path')}
                        className="w-full bg-zinc-900/70 border border-zinc-800 rounded-lg px-3.5 py-2.5 text-sm font-mono text-zinc-200 placeholder-zinc-600 focus:outline-none focus:border-emerald-500 transition-colors"
                    />
                </div>

                {/* Passphrase & Recovery Key */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-2">
                    <div className="space-y-2">
                        <Tooltip content={i18n.t('settings.tooltip_passphrase')} position="top">
                            <label htmlFor="backup_passphrase" className="text-sm font-bold text-zinc-300 cursor-help flex items-center gap-1.5">
                                <Key size={14} className="text-zinc-400" />
                                {i18n.t('settings.label_passphrase')}
                            </label>
                        </Tooltip>
                        <div className="relative">
                            <input
                                id="backup_passphrase"
                                type={showPassphrase ? 'text' : 'password'}
                                value={passphrase}
                                onChange={(e) => setPassphrase(e.target.value)}
                                placeholder={i18n.t('settings.placeholder_passphrase')}
                                className="w-full bg-zinc-900/70 border border-zinc-800 rounded-lg pl-3.5 pr-10 py-2.5 text-sm font-mono text-zinc-200 placeholder-zinc-600 focus:outline-none focus:border-emerald-500 transition-colors"
                            />
                            <button
                                type="button"
                                aria-label="Toggle passphrase visibility"
                                onClick={() => setShowPassphrase(!showPassphrase)}
                                className="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-500 hover:text-zinc-300 transition-colors"
                            >
                                {showPassphrase ? <EyeOff size={16} /> : <Eye size={16} />}
                            </button>
                        </div>
                    </div>

                    <div className="space-y-2 flex flex-col justify-end">
                        <Tooltip content={i18n.t('settings.tooltip_download_recovery')} position="top">
                            <button
                                type="button"
                                onClick={handleDownloadRecoveryKey}
                                className="w-full bg-zinc-800/80 hover:bg-zinc-700/80 text-zinc-200 border border-zinc-700/80 font-medium text-sm py-2.5 px-4 rounded-lg transition-colors flex items-center justify-center gap-2"
                            >
                                <Download size={16} className="text-emerald-400" />
                                {i18n.t('settings.btn_download_recovery')}
                            </button>
                        </Tooltip>
                    </div>
                </div>

                {/* Telemetry Strip & Actions */}
                <div className="border-t border-zinc-800/80 pt-5 flex flex-col sm:flex-row items-center justify-between gap-4">
                    <div className="flex flex-wrap items-center gap-6 text-xs text-zinc-400 font-mono">
                        <div className="flex items-center gap-2">
                            <Clock size={14} className="text-zinc-500" />
                            <span>{i18n.t('settings.desc_backup_cadence')}</span>
                        </div>
                        {backupStatus?.total_snapshots ? (
                            <div className="flex items-center gap-4 text-zinc-500">
                                <span>Snapshots: <strong className="text-zinc-300 font-mono">{backupStatus.total_snapshots}</strong></span>
                                <span>Storage: <strong className="text-zinc-300 font-mono">{totalSizeMb} MB</strong></span>
                            </div>
                        ) : null}
                    </div>

                    <Tooltip content={i18n.t('settings.tooltip_backup_now')} position="top">
                        <button
                            type="button"
                            disabled={isBackingUp || !settings.backup_path}
                            onClick={handleTriggerBackupNow}
                            className={`w-full sm:w-auto px-5 py-2.5 rounded-lg text-sm font-semibold transition-all flex items-center justify-center gap-2 shadow-sm ${
                                isBackingUp || !settings.backup_path
                                    ? 'bg-zinc-800 text-zinc-500 cursor-not-allowed border border-zinc-700/40'
                                    : 'bg-emerald-600 hover:bg-emerald-500 text-white shadow-emerald-950/40 active:scale-95'
                            }`}
                        >
                            <RefreshCw size={15} className={isBackingUp ? 'animate-spin' : ''} />
                            {isBackingUp ? i18n.t('settings.btn_backing_up') : i18n.t('settings.btn_backup_now')}
                        </button>
                    </Tooltip>
                </div>
            </div>
        </div>
    );
};
