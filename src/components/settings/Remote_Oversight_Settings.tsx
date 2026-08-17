/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Remote Oversight & Mobile Companion Settings Component**
 * Provides configuration for Zero-Trust remote access, QR pairing, and paired device management.
 * Decomposed with `PairedDevicesList`, `DevicePairingModal`, and `CompanionAuditLog` sub-components.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Pairing token generation failure, QR render desync, or remote device revocation failure.
 * - **Telemetry Link**: Search `[Remote_Oversight_Settings]` in observability traces.
 */

import React, { useState, useCallback, useEffect, useRef } from 'react';
import { ShieldCheck, Wifi, Radio, Globe } from 'lucide-react';
import { PairedDevicesList, type CompanionDevice } from './remote/PairedDevicesList';
import { CompanionAuditLog, type CompanionAuditEntry } from './remote/CompanionAuditLog';
import { DevicePairingModal } from './remote/DevicePairingModal';
import { useLocalStorage } from '../../hooks/useLocalStorage';
import { api_request } from '../../services/base_api_service';

interface RemoteDeviceDto {
    id: string;
    name: string;
    user_name?: string;
    public_key: string;
    paired_at: string;
    status: string;
}

interface RemoteDevicesResponse {
    devices: RemoteDeviceDto[];
}

interface PairingTokenResponse {
    token: string;
    expires_in_seconds: number;
}

const format_public_key = (public_key: string): string => {
    const normalized = public_key.replace(/^ed25519:/, '');
    if (normalized.length <= 12) return `ed25519:${normalized}`;
    return `ed25519:${normalized.slice(0, 6)}...${normalized.slice(-6)}`;
};

const map_remote_device = (device: RemoteDeviceDto): CompanionDevice => ({
    id: device.id,
    name: device.name,
    userName: device.user_name || 'Unassigned operator',
    pairedAt: device.paired_at,
    key: format_public_key(device.public_key),
    status: device.status || 'Authorized'
});

const describe_error = (error: unknown, fallback: string): string =>
    error instanceof Error && error.message ? error.message : fallback;

export const Remote_Oversight_Settings: React.FC = () => {
    const [is_bridge_enabled, set_is_bridge_enabled] = useLocalStorage<boolean>('tadpole_remote_bridge_enabled', true);
    const [tailscale_ip, set_tailscale_ip] = useLocalStorage<string>(
        'tadpole_remote_tailscale_ip',
        (import.meta.env.VITE_TAILSCALE_IP as string) || '10.0.0.1:8000'
    );
    const [lan_ip, set_lan_ip] = useLocalStorage<string>(
        'tadpole_remote_lan_ip',
        (import.meta.env.VITE_LAN_IP as string) || '10.0.0.1:8000'
    );
    const [pairing_mode, set_pairing_mode] = useLocalStorage<'lan' | 'tailscale'>('tadpole_remote_pairing_mode', 'lan');
    const [paired_devices, set_paired_devices] = useState<CompanionDevice[]>([]);
    const [audit_logs, set_audit_logs] = useLocalStorage<CompanionAuditEntry[]>('tadpole_remote_companion_audit_log', []);
    const [error_message, set_error_message] = useState<string>('');
    const pairing_baseline_ids = useRef<Set<string>>(new Set());

    // Pairing modal step state: 'closed' | 'config' | 'qr'
    const [pairing_modal_step, set_pairing_modal_step] = useState<'closed' | 'config' | 'qr'>('closed');
    const [pairing_token, set_pairing_token] = useState<string>('');

    // Input state for pairing request dialog
    const [device_name_input, set_device_name_input] = useState<string>('Android Companion (Pixel 8)');
    const [user_name_input, set_user_name_input] = useState<string>('Sovereign Operator');

    // Edit device modal state
    const [editing_device, set_editing_device] = useState<CompanionDevice | null>(null);
    const [edit_name_input, set_edit_name_input] = useState<string>('');
    const [edit_user_input, set_edit_user_input] = useState<string>('');

    const qr_target_ip = pairing_mode === 'lan' ? lan_ip : tailscale_ip;

    const get_formatted_timestamp = (): string => {
        const now = new Date();
        const yyyy = now.getFullYear();
        const mm = String(now.getMonth() + 1).padStart(2, '0');
        const dd = String(now.getDate()).padStart(2, '0');
        const hh = String(now.getHours()).padStart(2, '0');
        const min = String(now.getMinutes()).padStart(2, '0');
        return `${yyyy}-${mm}-${dd} ${hh}:${min}`;
    };

    const fetch_paired_devices = useCallback(async (): Promise<CompanionDevice[]> => {
        const response = await api_request<RemoteDevicesResponse>('/v1/remote/devices');
        const devices = response.devices.map(map_remote_device);
        set_paired_devices(devices);
        return devices;
    }, []);

    useEffect(() => {
        let is_active = true;
        void api_request<RemoteDevicesResponse>('/v1/remote/devices')
            .then(response => {
                if (is_active) set_paired_devices(response.devices.map(map_remote_device));
            })
            .catch((error: unknown) => {
                if (is_active) {
                    set_error_message(describe_error(error, 'Unable to load paired companion devices.'));
                }
            });
        return () => {
            is_active = false;
        };
    }, []);

    const handle_open_pairing_flow = useCallback(async () => {
        set_error_message('');
        try {
            const devices = await fetch_paired_devices();
            const response = await api_request<PairingTokenResponse>('/v1/remote/pairing-token');
            if (!response.token.startsWith('TP-PAIR-')) {
                throw new Error('The server returned an invalid pairing challenge.');
            }
            pairing_baseline_ids.current = new Set(devices.map(device => device.id));
            set_pairing_token(response.token);
            set_pairing_modal_step('config');
        } catch (error) {
            set_error_message(describe_error(error, 'Unable to create a server pairing challenge.'));
        }
    }, [fetch_paired_devices]);

    const handle_complete_pairing_scan = useCallback(async () => {
        set_error_message('');
        try {
            const devices = await fetch_paired_devices();
            const paired_device = devices.find(device => !pairing_baseline_ids.current.has(device.id));
            if (!paired_device) {
                set_error_message('No completed companion pairing was found. Scan the QR code, then refresh pairing status.');
                return;
            }

            const newAudit: CompanionAuditEntry = {
                id: `audit-${Date.now()}`,
                action: 'DEVICE_PAIRED',
                deviceName: paired_device.name,
                userName: paired_device.userName,
                key: paired_device.key,
                details: `Server confirmed QR pairing over ${pairing_mode.toUpperCase()} (${qr_target_ip})`,
                timestamp: get_formatted_timestamp()
            };
            set_audit_logs(prev => [newAudit, ...prev]);
            set_pairing_modal_step('closed');
        } catch (error) {
            set_error_message(describe_error(error, 'Unable to confirm companion pairing.'));
        }
    }, [fetch_paired_devices, pairing_mode, qr_target_ip, set_audit_logs]);

    const handle_edit_device = useCallback((device: CompanionDevice) => {
        set_editing_device(device);
        set_edit_name_input(device.name);
        set_edit_user_input(device.userName);
    }, []);

    const handle_save_edit_device = useCallback(async () => {
        if (!editing_device) return;
        const updatedName = edit_name_input.trim() || editing_device.name;
        const updatedUser = edit_user_input.trim() || editing_device.userName;
        set_error_message('');
        try {
            const updated = await api_request<RemoteDeviceDto>(`/v1/remote/devices/${encodeURIComponent(editing_device.id)}`, {
                method: 'PUT',
                body: JSON.stringify({ device_name: updatedName, user_name: updatedUser })
            });
            const authoritative_device = map_remote_device(updated);
            set_paired_devices(prev =>
                prev.map(device => device.id === authoritative_device.id ? authoritative_device : device)
            );
            set_audit_logs(prev => [{
                id: `audit-${Date.now()}`,
                action: 'DEVICE_EDITED',
                deviceName: authoritative_device.name,
                userName: authoritative_device.userName,
                key: authoritative_device.key,
                details: `Server confirmed metadata update from "${editing_device.name}" / "${editing_device.userName}"`,
                timestamp: get_formatted_timestamp()
            }, ...prev]);
            set_editing_device(null);
        } catch (error) {
            set_error_message(describe_error(error, 'Unable to update the paired device.'));
        }
    }, [editing_device, edit_name_input, edit_user_input, set_audit_logs]);

    const handle_revoke_device = useCallback(async (id: string) => {
        const target = paired_devices.find(d => d.id === id);
        if (!target) return;
        set_error_message('');
        try {
            await api_request(`/v1/remote/revoke/${encodeURIComponent(id)}`, { method: 'POST' });
            set_paired_devices(prev => prev.filter(device => device.id !== id));
            set_audit_logs(prev => [{
                id: `audit-${Date.now()}`,
                action: 'DEVICE_REVOKED',
                deviceName: target.name,
                userName: target.userName,
                key: target.key,
                details: 'Server confirmed companion access revocation',
                timestamp: get_formatted_timestamp()
            }, ...prev]);
        } catch (error) {
            set_error_message(describe_error(error, 'Unable to revoke the paired device.'));
        }
    }, [paired_devices, set_audit_logs]);

    const handle_clear_logs = useCallback(() => {
        set_audit_logs([]);
    }, [set_audit_logs]);

    return (
        <div className="space-y-6">
            <h2 className="text-sm font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2">
                <Radio className="w-4 h-4 text-emerald-400" />
                Remote Oversight & Mobile Mesh Settings
            </h2>

            {error_message && (
                <div role="alert" className="rounded-xl border border-rose-500/30 bg-rose-500/10 px-4 py-3 text-xs text-rose-300">
                    {error_message}
                </div>
            )}

            {/* Zero-Trust Remote Bridge Card */}
            <div className="bg-zinc-900/60 backdrop-blur-xl border border-white/5 rounded-2xl p-6 shadow-2xl space-y-5 relative overflow-hidden">
                <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-emerald-500/30 to-transparent" />

                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-zinc-800/80 pb-5">
                    <div className="flex items-center gap-3">
                        <div className="p-2 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400">
                            <ShieldCheck className="w-5 h-5" />
                        </div>
                        <div>
                            <h3 className="text-base font-semibold text-zinc-100 tracking-tight">
                                Zero-Trust Remote Companion Bridge
                            </h3>
                            <p className="text-xs text-zinc-400 mt-0.5">
                                Enables authenticated bidirectional RPC commands and push telemetry to authorized Android/iOS companion devices.
                            </p>
                        </div>
                    </div>

                    <div className="flex items-center gap-3">
                        <span className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-mono border ${
                            is_bridge_enabled
                                ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                                : 'bg-zinc-800 text-zinc-400 border-zinc-700'
                        }`}>
                            <span className={`w-1.5 h-1.5 rounded-full ${is_bridge_enabled ? 'bg-emerald-400 animate-pulse' : 'bg-zinc-500'}`} />
                            {is_bridge_enabled ? 'Active / Listening' : 'Disabled'}
                        </span>
                        <button
                            onClick={() => set_is_bridge_enabled(prev => !prev)}
                            className={`px-3 py-1.5 text-xs font-semibold rounded-xl border transition-all cursor-pointer ${
                                is_bridge_enabled
                                    ? 'bg-zinc-800 hover:bg-zinc-750 text-zinc-300 border-zinc-700'
                                    : 'bg-emerald-600 hover:bg-emerald-500 text-white border-emerald-500 shadow-lg shadow-emerald-950/40'
                            }`}
                        >
                            {is_bridge_enabled ? 'Disable Bridge' : 'Enable Bridge'}
                        </button>
                    </div>
                </div>

                {/* Network Pairing Mode & Endpoint Configuration */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-5 pt-1">
                    <div className="space-y-3">
                        <label className="block text-xs font-semibold text-zinc-400 uppercase tracking-wider">
                            Pairing Network Target
                        </label>
                        <div className="grid grid-cols-2 gap-2">
                            <button
                                onClick={() => set_pairing_mode('lan')}
                                className={`flex items-center justify-center gap-2 p-3 rounded-xl border text-xs font-semibold transition-all cursor-pointer ${
                                    pairing_mode === 'lan'
                                        ? 'bg-emerald-500/15 border-emerald-500/40 text-emerald-400 shadow-md shadow-emerald-950/20'
                                        : 'bg-zinc-950/40 border-zinc-800 text-zinc-400 hover:text-zinc-200 hover:border-zinc-700'
                                }`}
                            >
                                <Wifi className="w-4 h-4" />
                                <span>Local LAN Wi-Fi</span>
                            </button>
                            <button
                                onClick={() => set_pairing_mode('tailscale')}
                                className={`flex items-center justify-center gap-2 p-3 rounded-xl border text-xs font-semibold transition-all cursor-pointer ${
                                    pairing_mode === 'tailscale'
                                        ? 'bg-cyan-500/15 border-cyan-500/40 text-cyan-400 shadow-md shadow-cyan-950/20'
                                        : 'bg-zinc-950/40 border-zinc-800 text-zinc-400 hover:text-zinc-200 hover:border-zinc-700'
                                }`}
                            >
                                <Globe className="w-4 h-4" />
                                <span>Tailscale Mesh</span>
                            </button>
                        </div>
                    </div>

                    <div className="space-y-3">
                        <label className="block text-xs font-semibold text-zinc-400 uppercase tracking-wider">
                            Active Host Endpoint Address
                        </label>
                        <div className="relative">
                            <input
                                type="text"
                                value={pairing_mode === 'lan' ? lan_ip : tailscale_ip}
                                onChange={(e) => {
                                    if (pairing_mode === 'lan') set_lan_ip(e.target.value);
                                    else set_tailscale_ip(e.target.value);
                                }}
                                placeholder="e.g. 10.0.0.1:8000 or 10.0.0.1:8000"
                                className="w-full px-3.5 py-2.5 bg-zinc-950/40 border border-zinc-800 rounded-xl text-sm font-mono text-zinc-100 focus:outline-none focus:border-emerald-500/50 focus:ring-1 focus:ring-emerald-500/30 transition-all"
                            />
                        </div>
                    </div>
                </div>
            </div>

            {/* Paired Devices List Sub-Component */}
            <PairedDevicesList
                devices={paired_devices}
                onOpenPairing={handle_open_pairing_flow}
                onEditDevice={handle_edit_device}
                onRevokeDevice={handle_revoke_device}
            />

            {/* Companion Security Audit Log Sub-Component */}
            <CompanionAuditLog
                logs={audit_logs}
                onClearLogs={handle_clear_logs}
            />

            {/* Modals Sub-Component */}
            <DevicePairingModal
                pairingStep={pairing_modal_step}
                pairingToken={pairing_token}
                qrTargetIp={qr_target_ip}
                deviceNameInput={device_name_input}
                userNameInput={user_name_input}
                onDeviceNameChange={set_device_name_input}
                onUserNameChange={set_user_name_input}
                onClosePairing={() => set_pairing_modal_step('closed')}
                onProceedToQr={() => set_pairing_modal_step('qr')}
                onBackToConfig={() => set_pairing_modal_step('config')}
                onCompletePairing={handle_complete_pairing_scan}
                editingDevice={editing_device}
                editNameInput={edit_name_input}
                editUserInput={edit_user_input}
                onEditNameChange={set_edit_name_input}
                onEditUserChange={set_edit_user_input}
                onCloseEdit={() => set_editing_device(null)}
                onSaveEdit={handle_save_edit_device}
            />
        </div>
    );
};

// Metadata: [Remote_Oversight_Settings]
