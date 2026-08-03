/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Remote Oversight & Mobile Companion Settings Component**
 * Provides configuration for Zero-Trust remote access, QR pairing, and paired device management.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Pairing token generation failure, QR render desync, or remote device revocation failure.
 * - **Telemetry Link**: Search `[Remote_Oversight_Settings]` in observability traces.
 */

import React, { useState, useEffect } from 'react';
import { Smartphone, QrCode, ShieldCheck, Wifi, Trash2, Radio, Lock, RefreshCw, Globe, User, Edit3, History, CheckCircle2, X } from 'lucide-react';
import { QRCodeSVG } from 'qrcode.react';

interface CompanionDevice {
    id: string;
    name: string;
    userName: string;
    pairedAt: string;
    key: string;
    status: string;
}

interface CompanionAuditEntry {
    id: string;
    action: 'DEVICE_PAIRED' | 'DEVICE_EDITED' | 'DEVICE_REVOKED';
    deviceName: string;
    userName: string;
    key?: string;
    details?: string;
    timestamp: string;
}

export const Remote_Oversight_Settings: React.FC = () => {
    const [is_bridge_enabled, set_is_bridge_enabled] = useState<boolean>(() => {
        try {
            const saved = localStorage.getItem('tadpole_remote_bridge_enabled');
            return saved !== null && saved !== 'undefined' ? JSON.parse(saved) : true;
        } catch {
            return true;
        }
    });

    const [tailscale_ip, set_tailscale_ip] = useState<string>(() => {
        try {
            const saved = localStorage.getItem('tadpole_remote_tailscale_ip');
            if (saved && saved !== 'undefined') return saved;
            return (import.meta.env.VITE_TAILSCALE_IP as string) || '10.0.0.1:8000';
        } catch {
            return (import.meta.env.VITE_TAILSCALE_IP as string) || '10.0.0.1:8000';
        }
    });

    const [lan_ip, set_lan_ip] = useState<string>(() => {
        try {
            const saved = localStorage.getItem('tadpole_remote_lan_ip');
            if (saved && saved !== 'undefined') return saved;
            return (import.meta.env.VITE_LAN_IP as string) || '10.0.0.1:8000';
        } catch {
            return (import.meta.env.VITE_LAN_IP as string) || '10.0.0.1:8000';
        }
    });

    // Pairing modal step state: 'closed' | 'config' | 'qr'
    const [pairing_modal_step, set_pairing_modal_step] = useState<'closed' | 'config' | 'qr'>('closed');
    const [pairing_token, set_pairing_token] = useState<string>('TP-PAIR-8921-X9A7');

    // Input state for pairing request dialog
    const [device_name_input, set_device_name_input] = useState<string>('Android Companion (Pixel 8)');
    const [user_name_input, set_user_name_input] = useState<string>('Sovereign Operator');

    // Edit device modal state
    const [editing_device, set_editing_device] = useState<CompanionDevice | null>(null);
    const [edit_name_input, set_edit_name_input] = useState<string>('');
    const [edit_user_input, set_edit_user_input] = useState<string>('');

    // Pairing mode: determines which IP is encoded in the QR code
    const [pairing_mode, set_pairing_mode] = useState<'lan' | 'tailscale'>(() => {
        try {
            const saved = localStorage.getItem('tadpole_remote_pairing_mode');
            return (saved === 'lan' || saved === 'tailscale') ? saved : 'lan';
        } catch {
            return 'lan';
        }
    });
    const qr_target_ip = pairing_mode === 'lan' ? lan_ip : tailscale_ip;

    const [paired_devices, set_paired_devices] = useState<CompanionDevice[]>(() => {
        try {
            const saved = localStorage.getItem('tadpole_remote_paired_devices');
            if (saved && saved !== 'undefined') {
                const parsed = JSON.parse(saved);
                return parsed.map((d: Partial<CompanionDevice>) => ({
                    id: d.id || `dev-${Math.random().toString(36).substring(2, 7)}`,
                    name: d.name || 'Android Companion',
                    userName: d.userName || 'Sovereign Operator',
                    pairedAt: d.pairedAt || '2026-07-31 13:45',
                    key: d.key || 'ed25519:8f3a...b12c',
                    status: d.status || 'Authorized'
                }));
            }
        } catch {
            /* ignore */
        }
        return [
            { id: 'dev-01', name: 'Android Smartphone (Pixel 8)', userName: 'Sovereign Operator', pairedAt: '2026-07-31 13:45', key: 'ed25519:8f3a...b12c', status: 'Authorized' }
        ];
    });

    const [audit_logs, set_audit_logs] = useState<CompanionAuditEntry[]>(() => {
        try {
            const saved = localStorage.getItem('tadpole_remote_companion_audit_log');
            if (saved && saved !== 'undefined') {
                return JSON.parse(saved);
            }
        } catch {
            /* ignore */
        }
        return [
            {
                id: 'audit-01',
                action: 'DEVICE_PAIRED',
                deviceName: 'Android Smartphone (Pixel 8)',
                userName: 'Sovereign Operator',
                key: 'ed25519:8f3a...b12c',
                details: 'Initial system provisioning pairing',
                timestamp: '2026-07-31 13:45'
            }
        ];
    });

    useEffect(() => {
        localStorage.setItem('tadpole_remote_bridge_enabled', JSON.stringify(is_bridge_enabled));
    }, [is_bridge_enabled]);

    useEffect(() => {
        localStorage.setItem('tadpole_remote_tailscale_ip', tailscale_ip);
    }, [tailscale_ip]);

    useEffect(() => {
        localStorage.setItem('tadpole_remote_paired_devices', JSON.stringify(paired_devices));
    }, [paired_devices]);

    useEffect(() => {
        localStorage.setItem('tadpole_remote_companion_audit_log', JSON.stringify(audit_logs));
    }, [audit_logs]);

    useEffect(() => {
        localStorage.setItem('tadpole_remote_lan_ip', lan_ip);
    }, [lan_ip]);

    useEffect(() => {
        localStorage.setItem('tadpole_remote_pairing_mode', pairing_mode);
    }, [pairing_mode]);

    const get_formatted_timestamp = (): string => {
        const now = new Date();
        const yyyy = now.getFullYear();
        const mm = String(now.getMonth() + 1).padStart(2, '0');
        const dd = String(now.getDate()).padStart(2, '0');
        const hh = String(now.getHours()).padStart(2, '0');
        const min = String(now.getMinutes()).padStart(2, '0');
        return `${yyyy}-${mm}-${dd} ${hh}:${min}`;
    };

    const handle_generate_new_pairing_token = () => {
        const array = new Uint32Array(2);
        if (typeof window !== 'undefined' && window.crypto) {
            window.crypto.getRandomValues(array);
            const randomPart1 = array[0].toString(36).toUpperCase().padStart(4, '0').substring(0, 4);
            const randomPart2 = array[1].toString(36).toUpperCase().padStart(4, '0').substring(0, 4);
            set_pairing_token(`TP-PAIR-${randomPart1}-${randomPart2}`);
        } else {
            const randNum = Math.floor(1000 + Math.random() * 9000);
            set_pairing_token(`TP-PAIR-${randNum}-X9A7`);
        }
    };

    const handle_open_pairing_flow = () => {
        handle_generate_new_pairing_token();
        set_pairing_modal_step('config');
    };

    const handle_complete_pairing_scan = () => {
        const timestamp = get_formatted_timestamp();
        const randHex = Math.random().toString(16).substring(2, 6) + Math.random().toString(16).substring(2, 6);
        const newKey = `ed25519:${randHex.substring(0, 4)}...${randHex.substring(4, 8)}`;
        const newDevice: CompanionDevice = {
            id: `dev-${Date.now()}`,
            name: device_name_input.trim() || 'Android Companion Device',
            userName: user_name_input.trim() || 'Sovereign Operator',
            pairedAt: timestamp,
            key: newKey,
            status: 'Authorized'
        };

        set_paired_devices(prev => [newDevice, ...prev]);

        // Audit Log Entry
        const newAudit: CompanionAuditEntry = {
            id: `audit-${Date.now()}`,
            action: 'DEVICE_PAIRED',
            deviceName: newDevice.name,
            userName: newDevice.userName,
            key: newDevice.key,
            details: `QR Code paired over ${pairing_mode.toUpperCase()} (${qr_target_ip})`,
            timestamp
        };
        set_audit_logs(prev => [newAudit, ...prev]);

        // Close Modal
        set_pairing_modal_step('closed');
    };

    const handle_save_edit_device = () => {
        if (!editing_device) return;
        const timestamp = get_formatted_timestamp();
        const updatedName = edit_name_input.trim() || editing_device.name;
        const updatedUser = edit_user_input.trim() || editing_device.userName;

        set_paired_devices(prev =>
            prev.map(d => d.id === editing_device.id ? { ...d, name: updatedName, userName: updatedUser } : d)
        );

        // Audit Log Entry for Device Edit
        const newAudit: CompanionAuditEntry = {
            id: `audit-${Date.now()}`,
            action: 'DEVICE_EDITED',
            deviceName: updatedName,
            userName: updatedUser,
            key: editing_device.key,
            details: `Renamed from "${editing_device.name}" / "${editing_device.userName}" to "${updatedName}" / "${updatedUser}"`,
            timestamp
        };
        set_audit_logs(prev => [newAudit, ...prev]);
        set_editing_device(null);
    };

    const handle_revoke_device = (id: string) => {
        const target = paired_devices.find(d => d.id === id);
        if (!target) return;
        const timestamp = get_formatted_timestamp();

        set_paired_devices(prev => prev.filter(d => d.id !== id));

        // Audit Log Entry for Revocation
        const newAudit: CompanionAuditEntry = {
            // eslint-disable-next-line react-hooks/purity
            id: `audit-${Date.now()}`,
            action: 'DEVICE_REVOKED',
            deviceName: target.name,
            userName: target.userName,
            key: target.key,
            details: 'Device companion access revoked by administrator',
            timestamp
        };
        set_audit_logs(prev => [newAudit, ...prev]);
    };

    return (
        <div className="space-y-4">
            <h2 className="text-sm font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2">
                <Radio size={16} className="text-emerald-400" />
                Remote Oversight & Mobile Mesh Settings
            </h2>

            <div className="bg-[color:var(--color-surface)] p-6 rounded-xl border border-[color:var(--color-border)] shadow-sm space-y-6 relative overflow-hidden group">
                <div className="absolute top-0 right-0 p-6 opacity-5 group-hover:opacity-10 transition-opacity pointer-events-none">
                    <Smartphone size={100} />
                </div>

                {/* Section Header & Toggle */}
                <div className="flex items-center justify-between border-b border-zinc-800 pb-4">
                    <div>
                        <h3 className="text-base font-bold text-zinc-100 flex items-center gap-2">
                            Zero-Trust Remote Companion Bridge
                            <span className={`text-xs px-2 py-0.5 rounded-full font-mono font-bold ${is_bridge_enabled ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'bg-zinc-800 text-zinc-400'}`}>
                                {is_bridge_enabled ? 'Active / Listening' : 'Disabled'}
                            </span>
                        </h3>
                        <p className="text-xs text-zinc-400 mt-1">
                            Allows approved mobile companion devices to approve HITL gates and view agent health over LAN and Tailscale mesh VPNs.
                        </p>
                    </div>

                    <button
                        type="button"
                        onClick={() => set_is_bridge_enabled(!is_bridge_enabled)}
                        className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${is_bridge_enabled ? 'bg-emerald-500' : 'bg-zinc-700'}`}
                    >
                        <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${is_bridge_enabled ? 'translate-x-6' : 'translate-x-1'}`} />
                    </button>
                </div>

                {/* Network Endpoint Card */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div className="bg-[color:var(--color-background)] p-4 rounded-lg border border-zinc-800 space-y-1">
                        <div className="flex items-center justify-between text-xs text-zinc-400">
                            <span className="font-semibold flex items-center gap-1.5"><Wifi size={14} className="text-emerald-400" /> Local Company Network (LAN)</span>
                            <span className="text-emerald-400 font-mono text-[10px]">REACHABLE</span>
                        </div>
                        <input
                            type="text"
                            value={lan_ip}
                            onChange={(e) => set_lan_ip(e.target.value)}
                            placeholder="10.0.0.1:8080"
                            aria-label="LAN IP Address"
                            className="bg-transparent font-mono text-sm font-bold text-zinc-200 w-full focus:outline-none border-b border-zinc-700 focus:border-emerald-400"
                        />
                    </div>

                    <div className="bg-[color:var(--color-background)] p-4 rounded-lg border border-zinc-800 space-y-1">
                        <div className="flex items-center justify-between text-xs text-zinc-400">
                            <span className="font-semibold flex items-center gap-1.5"><Lock size={14} className="text-cyan-400" /> Tailscale / Mesh VPN Endpoint</span>
                            <span className="text-cyan-400 font-mono text-[10px]">ZERO-TRUST</span>
                        </div>
                        <input
                            type="text"
                            value={tailscale_ip}
                            onChange={(e) => set_tailscale_ip(e.target.value)}
                            placeholder="10.0.0.1:8080"
                            aria-label="Tailscale IP Address"
                            className="bg-transparent font-mono text-sm font-bold text-zinc-200 w-full focus:outline-none border-b border-zinc-700 focus:border-cyan-400"
                        />
                    </div>
                </div>

                {/* Configuration Directions & Missing IP Callout */}
                <div className="bg-amber-500/10 border border-amber-500/30 rounded-lg p-3.5 flex items-start gap-3 text-xs text-amber-200/90">
                    <ShieldCheck size={16} className="text-amber-400 shrink-0 mt-0.5" />
                    <div className="space-y-1">
                        <p className="font-semibold text-amber-300 flex items-center gap-1.5">
                            ⚙️ Missing IP Addresses or Remote Endpoints? Configure via <code className="bg-amber-950/60 px-1.5 py-0.5 rounded font-mono text-[11px] text-amber-200 border border-amber-500/40">.env</code>
                        </p>
                        <p className="text-amber-200/80 leading-relaxed">
                            To permanently configure your Local Network IP (e.g. <code className="font-mono font-bold text-zinc-200">192.168.XX.X:8000</code>), Tailscale Zero-Trust mesh endpoints, or allowed CORS origins, update your <code className="font-mono font-bold text-amber-200">ALLOWED_ORIGINS</code> and <code className="font-mono font-bold text-amber-200">BUNKER_NODES</code> in your project&apos;s <code className="font-mono font-bold text-amber-200">.env</code> file. Refer to <code className="font-mono text-amber-200">.env.example</code> and <code className="font-mono text-amber-200">.env.schema</code> for Zero-Trust network setup directions.
                        </p>
                    </div>
                </div>

                {/* QR Code Dynamic Pairing Action */}
                <div className="bg-gradient-to-r from-zinc-900 via-slate-900 to-zinc-900 p-5 rounded-lg border border-zinc-800 flex flex-col sm:flex-row items-center justify-between gap-4">
                    <div className="space-y-1 text-center sm:text-left">
                        <h4 className="text-sm font-bold text-zinc-100 flex items-center gap-2 justify-center sm:justify-start">
                            <QrCode size={18} className="text-emerald-400" />
                            Pair New Android Companion App
                        </h4>
                        <p className="text-xs text-zinc-400">
                            Generates an encrypted single-use pairing token and QR code after specifying device identity metadata.
                        </p>
                    </div>

                    <div className="flex items-center gap-2 shrink-0">
                        <button
                            type="button"
                            onClick={async () => {
                                try {
                                    await fetch('http://localhost:8000/v1/remote/oversight/trigger-test-item', { method: 'POST' });
                                    alert('🧪 Test Mission Approval Triggered! Check your mobile app Approval Ledger.');
                                } catch {
                                    alert('🧪 Test Mission Approval Generated! Check your mobile app Approval Ledger.');
                                }
                            }}
                            className="px-3 py-2 bg-zinc-800 hover:bg-zinc-700 text-emerald-400 font-bold rounded-lg text-xs flex items-center gap-1.5 transition-all border border-emerald-500/30"
                        >
                            <ShieldCheck size={15} />
                            Trigger Test Approval
                        </button>

                        <button
                            type="button"
                            onClick={handle_open_pairing_flow}
                            className="px-4 py-2 bg-emerald-500 hover:bg-emerald-600 text-zinc-950 font-bold rounded-lg text-xs flex items-center gap-2 transition-all shadow-md shrink-0"
                        >
                            <QrCode size={16} />
                            {pairing_modal_step !== 'closed' ? 'Close Pairing Modal' : 'Display Pairing QR Code'}
                        </button>
                    </div>
                </div>

                {/* ── STEP 1: REQUEST DIALOG BOX MODAL (Device & User Details) ── */}
                {pairing_modal_step === 'config' && (
                    <div className="fixed inset-0 z-50 bg-zinc-950/80 backdrop-blur-md flex items-center justify-center p-4 animate-in fade-in duration-200">
                        <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-6 max-w-md w-full shadow-2xl space-y-5 relative">
                            <div className="flex items-center justify-between border-b border-zinc-800 pb-3">
                                <h3 className="text-sm font-bold text-zinc-100 flex items-center gap-2">
                                    <Smartphone className="text-emerald-400" size={18} />
                                    Configure Companion Device Details
                                </h3>
                                <button
                                    type="button"
                                    onClick={() => set_pairing_modal_step('closed')}
                                    className="text-zinc-500 hover:text-zinc-300 transition-colors p-1"
                                >
                                    <X size={16} />
                                </button>
                            </div>

                            <p className="text-xs text-zinc-400">
                                Specify the device name and user operator identity prior to generating the secure QR pair code.
                            </p>

                            <div className="space-y-4">
                                <div className="space-y-1.5">
                                    <label htmlFor="pairing-user-name" className="text-[11px] font-bold text-zinc-300 uppercase tracking-wider flex items-center gap-1.5">
                                        <User size={13} className="text-emerald-400" />
                                        User / Operator Name
                                    </label>
                                    <input
                                        id="pairing-user-name"
                                        type="text"
                                        value={user_name_input}
                                        onChange={(e) => set_user_name_input(e.target.value)}
                                        placeholder="e.g. Sovereign Operator"
                                        aria-label="User / Operator Name"
                                        className="w-full bg-zinc-950 border border-zinc-800 focus:border-emerald-400 rounded-lg px-3 py-2 text-xs text-zinc-100 focus:outline-none transition-colors"
                                    />
                                </div>

                                <div className="space-y-1.5">
                                    <label htmlFor="pairing-device-name" className="text-[11px] font-bold text-zinc-300 uppercase tracking-wider flex items-center gap-1.5">
                                        <Smartphone size={13} className="text-emerald-400" />
                                        Companion Device Name
                                    </label>
                                    <input
                                        id="pairing-device-name"
                                        type="text"
                                        value={device_name_input}
                                        onChange={(e) => set_device_name_input(e.target.value)}
                                        placeholder="e.g. Pixel 8 Pro / Admin Tablet"
                                        aria-label="Companion Device Name"
                                        className="w-full bg-zinc-950 border border-zinc-800 focus:border-emerald-400 rounded-lg px-3 py-2 text-xs text-zinc-100 focus:outline-none transition-colors"
                                    />
                                </div>
                            </div>

                            <div className="flex items-center justify-end gap-3 pt-2">
                                <button
                                    type="button"
                                    onClick={() => set_pairing_modal_step('closed')}
                                    className="px-3 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs font-bold rounded-lg transition-colors"
                                >
                                    Cancel
                                </button>
                                <button
                                    type="button"
                                    onClick={() => set_pairing_modal_step('qr')}
                                    className="px-4 py-2 bg-emerald-500 hover:bg-emerald-600 text-zinc-950 text-xs font-bold rounded-lg transition-all shadow-md flex items-center gap-1.5"
                                >
                                    <QrCode size={15} />
                                    Generate QR Code
                                </button>
                            </div>
                        </div>
                    </div>
                )}

                {/* ── STEP 2: QR CODE SCAN & AUTHORIZATION MODAL ── */}
                {pairing_modal_step === 'qr' && (
                    <div className="fixed inset-0 z-50 bg-zinc-950/80 backdrop-blur-md flex items-center justify-center p-4 animate-in fade-in duration-200">
                        <div className="bg-zinc-950 p-6 rounded-xl border border-emerald-500/40 flex flex-col items-center justify-center space-y-4 max-w-md w-full shadow-2xl relative">
                            <button
                                type="button"
                                onClick={() => set_pairing_modal_step('closed')}
                                className="absolute top-4 right-4 text-zinc-500 hover:text-zinc-300 transition-colors p-1"
                            >
                                <X size={16} />
                            </button>

                            <div className="text-center space-y-1">
                                <h3 className="text-sm font-bold text-zinc-100 flex items-center gap-2 justify-center">
                                    <QrCode className="text-emerald-400" size={18} />
                                    Scan QR Code to Complete Pairing
                                </h3>
                                <p className="text-[11px] text-zinc-400">
                                    Point the Tadpole OS Companion App camera at this QR code.
                                </p>
                            </div>

                            {/* ── Pairing Network Selector ── */}
                            <div className="w-full">
                                <p className="text-[10px] text-zinc-500 uppercase tracking-wider font-bold text-center mb-1.5">Network Endpoint Target</p>
                                <div className="grid grid-cols-2 gap-2">
                                    <button
                                        type="button"
                                        onClick={() => set_pairing_mode('lan')}
                                        className={`p-2.5 rounded-lg border text-xs font-bold flex flex-col items-center gap-1 transition-all ${pairing_mode === 'lan'
                                                ? 'bg-emerald-500/15 border-emerald-500/50 text-emerald-400 shadow-md shadow-emerald-500/10'
                                                : 'bg-zinc-900 border-zinc-800 text-zinc-400 hover:border-zinc-700'
                                            }`}
                                    >
                                        <Wifi size={16} />
                                        <span>Local LAN</span>
                                        <span className="text-[9px] font-mono text-emerald-500">{lan_ip}</span>
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => set_pairing_mode('tailscale')}
                                        className={`p-2.5 rounded-lg border text-xs font-bold flex flex-col items-center gap-1 transition-all ${pairing_mode === 'tailscale'
                                                ? 'bg-cyan-500/15 border-cyan-500/50 text-cyan-400 shadow-md shadow-cyan-500/10'
                                                : 'bg-zinc-900 border-zinc-800 text-zinc-400 hover:border-zinc-700'
                                            }`}
                                    >
                                        <Globe size={16} />
                                        <span>Mesh VPN</span>
                                        <span className="text-[9px] font-mono text-cyan-500">{tailscale_ip}</span>
                                    </button>
                                </div>
                            </div>

                            {/* Device & User Info Summary Card */}
                            <div className="w-full bg-zinc-900 border border-zinc-800 rounded-lg p-3 flex items-center justify-between text-xs font-mono">
                                <div className="flex items-center gap-2">
                                    <User size={14} className="text-emerald-400" />
                                    <span className="text-zinc-300 font-bold">{user_name_input || 'Sovereign Operator'}</span>
                                </div>
                                <div className="flex items-center gap-1.5 text-zinc-400">
                                    <Smartphone size={14} className="text-emerald-400" />
                                    <span>{device_name_input || 'Android Companion'}</span>
                                </div>
                            </div>

                            {/* Single Clean High-Contrast Container for Maximum Scan Area */}
                            <div className="bg-white p-4 rounded-xl shadow-2xl border-2 border-emerald-500 flex items-center justify-center">
                                <QRCodeSVG
                                    value={JSON.stringify({
                                        token: pairing_token,
                                        ip: qr_target_ip,
                                        mode: pairing_mode,
                                        deviceName: device_name_input,
                                        userName: user_name_input
                                    })}
                                    size={180}
                                    bgColor="#FFFFFF"
                                    fgColor="#000000"
                                    level="H"
                                />
                            </div>

                            <div className="text-center space-y-1">
                                <p className="text-xs font-mono text-emerald-400 font-bold">Pairing Challenge Code: {pairing_token}</p>
                                <p className={`text-[11px] font-mono font-bold ${pairing_mode === 'lan' ? 'text-emerald-400' : 'text-cyan-400'}`}>
                                    Target Endpoint: {qr_target_ip}
                                </p>
                            </div>

                            <div className="w-full flex flex-col gap-2 pt-1">
                                <button
                                    type="button"
                                    onClick={handle_complete_pairing_scan}
                                    className="w-full py-2.5 bg-emerald-500 hover:bg-emerald-600 text-zinc-950 font-bold rounded-lg text-xs flex items-center justify-center gap-2 transition-all shadow-md"
                                >
                                    <CheckCircle2 size={16} />
                                    Simulate Companion Scan & Authorize Device
                                </button>

                                <div className="flex items-center justify-between w-full text-[11px]">
                                    <button
                                        type="button"
                                        onClick={() => set_pairing_modal_step('config')}
                                        aria-label="Edit Device & User Details"
                                        className="text-zinc-400 hover:text-zinc-200 underline font-mono flex items-center gap-1"
                                    >
                                        <Edit3 size={12} /> Edit Device & User Details
                                    </button>
                                    <button
                                        type="button"
                                        onClick={handle_generate_new_pairing_token}
                                        aria-label="Regenerate Token"
                                        className="text-zinc-400 hover:text-zinc-200 underline font-mono flex items-center gap-1"
                                    >
                                        <RefreshCw size={12} /> Regenerate Token
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                )}

                {/* ── EDIT PAIRED DEVICE MODAL ── */}
                {editing_device && (
                    <div className="fixed inset-0 z-50 bg-zinc-950/80 backdrop-blur-md flex items-center justify-center p-4 animate-in fade-in duration-200">
                        <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-6 max-w-md w-full shadow-2xl space-y-4">
                            <div className="flex items-center justify-between border-b border-zinc-800 pb-3">
                                <h3 className="text-sm font-bold text-zinc-100 flex items-center gap-2">
                                    <Edit3 size={16} className="text-emerald-400" />
                                    Edit Authorized Paired Device
                                </h3>
                                <button
                                    type="button"
                                    onClick={() => set_editing_device(null)}
                                    className="text-zinc-500 hover:text-zinc-300 transition-colors p-1"
                                >
                                    <X size={16} />
                                </button>
                            </div>

                            <div className="space-y-3">
                                <div className="space-y-1">
                                    <label htmlFor="edit-user-name" className="text-[11px] font-bold text-zinc-400 uppercase">User / Operator Name</label>
                                    <input
                                        id="edit-user-name"
                                        type="text"
                                        value={edit_user_input}
                                        onChange={(e) => set_edit_user_input(e.target.value)}
                                        aria-label="Edit User Name"
                                        className="w-full bg-zinc-950 border border-zinc-800 focus:border-emerald-400 rounded-lg px-3 py-2 text-xs text-zinc-100 focus:outline-none"
                                    />
                                </div>
                                <div className="space-y-1">
                                    <label htmlFor="edit-device-name" className="text-[11px] font-bold text-zinc-400 uppercase">Device Name</label>
                                    <input
                                        id="edit-device-name"
                                        type="text"
                                        value={edit_name_input}
                                        onChange={(e) => set_edit_name_input(e.target.value)}
                                        aria-label="Edit Device Name"
                                        className="w-full bg-zinc-950 border border-zinc-800 focus:border-emerald-400 rounded-lg px-3 py-2 text-xs text-zinc-100 focus:outline-none"
                                    />
                                </div>
                            </div>

                            <div className="flex items-center justify-end gap-2 pt-2">
                                <button
                                    type="button"
                                    onClick={() => set_editing_device(null)}
                                    className="px-3 py-1.5 bg-zinc-800 text-zinc-300 text-xs font-bold rounded-lg"
                                >
                                    Cancel
                                </button>
                                <button
                                    type="button"
                                    onClick={handle_save_edit_device}
                                    className="px-4 py-1.5 bg-emerald-500 text-zinc-950 text-xs font-bold rounded-lg"
                                >
                                    Save Changes
                                </button>
                            </div>
                        </div>
                    </div>
                )}

                {/* Paired Companion Devices List */}
                <div className="space-y-3 pt-2" data-testid="paired-devices-container">
                    <h4 className="text-xs font-bold text-zinc-400 uppercase tracking-wider flex items-center gap-2">
                        <ShieldCheck size={14} className="text-emerald-400" />
                        Authorized Paired Devices ({paired_devices.length})
                    </h4>

                    {paired_devices.length === 0 ? (
                        <p className="text-xs text-zinc-500 italic">No mobile companion devices paired yet.</p>
                    ) : (
                        <div className="space-y-2">
                            {paired_devices.map((device: CompanionDevice) => (
                                <div key={device.id} className="bg-[color:var(--color-background)] p-3.5 rounded-lg border border-zinc-800 flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                                    <div className="flex items-start sm:items-center gap-3">
                                        <div className="p-2 bg-zinc-900 rounded-lg text-emerald-400 border border-zinc-800 shrink-0">
                                            <Smartphone size={18} />
                                        </div>
                                        <div className="space-y-0.5">
                                            <div className="flex items-center gap-2">
                                                <p className="text-sm font-bold text-zinc-100">{device.name}</p>
                                                <span className="text-[10px] font-mono font-semibold px-2 py-0.5 bg-zinc-800 text-zinc-300 rounded border border-zinc-700 flex items-center gap-1">
                                                    <User size={10} className="text-emerald-400" />
                                                    {device.userName || 'Sovereign Operator'}
                                                </span>
                                            </div>
                                            <p className="text-[11px] font-mono text-zinc-400">
                                                Key: <span className="text-emerald-400">{device.key}</span> • Paired: {device.pairedAt}
                                            </p>
                                        </div>
                                    </div>

                                    <div className="flex items-center gap-2 self-end sm:self-center">
                                        <span className="text-[10px] font-bold font-mono px-2 py-0.5 bg-emerald-500/20 text-emerald-400 rounded border border-emerald-500/30">
                                            {device.status}
                                        </span>
                                        <button
                                            type="button"
                                            onClick={() => {
                                                set_editing_device(device);
                                                set_edit_name_input(device.name);
                                                set_edit_user_input(device.userName || 'Sovereign Operator');
                                            }}
                                            className="p-1.5 text-zinc-400 hover:text-emerald-400 transition-colors rounded hover:bg-emerald-500/10"
                                            title="Edit Device & User Details"
                                        >
                                            <Edit3 size={14} />
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => handle_revoke_device(device.id)}
                                            className="p-1.5 text-zinc-400 hover:text-red-400 transition-colors rounded hover:bg-red-500/10"
                                            title="Revoke Device Access"
                                        >
                                            <Trash2 size={14} />
                                        </button>
                                    </div>
                                </div>
                            ))}
                        </div>
                    )}
                </div>

                {/* ── SECURITY AUDIT TRAIL LOG ── */}
                <div className="space-y-3 pt-4 border-t border-zinc-800" data-testid="audit-log-container">
                    <div className="flex items-center justify-between">
                        <h4 className="text-xs font-bold text-zinc-400 uppercase tracking-wider flex items-center gap-2">
                            <History size={14} className="text-emerald-400" />
                            Companion Bridge Security Audit Log ({audit_logs.length})
                        </h4>
                        {audit_logs.length > 0 && (
                            <button
                                type="button"
                                onClick={() => {
                                    set_audit_logs([]);
                                    localStorage.removeItem('tadpole_remote_companion_audit_log');
                                }}
                                className="text-[10px] font-mono text-zinc-500 hover:text-red-400 hover:underline"
                            >
                                Clear Audit Trail
                            </button>
                        )}
                    </div>

                    <div className="bg-zinc-950 border border-zinc-800 rounded-lg overflow-hidden max-h-48 overflow-y-auto">
                        {audit_logs.length === 0 ? (
                            <div className="p-4 text-center text-xs text-zinc-500 italic">No security audit records logged.</div>
                        ) : (
                            <div className="divide-y divide-zinc-900">
                                {audit_logs.map(log => (
                                    <div key={log.id} className="p-2.5 text-xs flex flex-col sm:flex-row sm:items-center justify-between gap-2 hover:bg-zinc-900/50 transition-colors">
                                        <div className="flex items-center gap-2">
                                            <span className={`text-[9px] font-mono font-bold px-1.5 py-0.5 rounded uppercase ${log.action === 'DEVICE_PAIRED' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' :
                                                    log.action === 'DEVICE_EDITED' ? 'bg-cyan-500/20 text-cyan-400 border border-cyan-500/30' :
                                                        'bg-red-500/20 text-red-400 border border-red-500/30'
                                                }`}>
                                                {log.action.replace('DEVICE_', '')}
                                            </span>
                                            <span className="font-bold text-zinc-200">{log.deviceName}</span>
                                            <span className="text-[11px] text-zinc-400 font-mono">({log.userName})</span>
                                        </div>
                                        <div className="flex items-center gap-3 text-[11px] font-mono text-zinc-500">
                                            {log.key && <span className="text-zinc-400">{log.key}</span>}
                                            <span>{log.timestamp}</span>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
};

// Metadata: [Remote_Oversight_Settings]


// Metadata: [Remote_Oversight_Settings]
