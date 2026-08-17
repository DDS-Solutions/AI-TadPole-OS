/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Paired Devices List Component**
 * Displays table of authorized mobile companion devices with cryptographic Ed25519 keys,
 * pairing timestamps, status badges, and edit/revoke actions adhering to design.md.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Missing device key render or state mutation error.
 * - **Telemetry Link**: Search `[PairedDevicesList]` in UI logs.
 */

import React, { memo } from 'react';
import { Smartphone, QrCode, Trash2, Edit3, ShieldCheck } from 'lucide-react';
import { Tooltip } from '../../ui';

export interface CompanionDevice {
    id: string;
    name: string;
    userName: string;
    pairedAt: string;
    key: string;
    status: string;
}

export interface PairedDevicesListProps {
    devices: CompanionDevice[];
    onOpenPairing: () => void | Promise<void>;
    onEditDevice: (device: CompanionDevice) => void;
    onRevokeDevice: (deviceId: string) => void | Promise<void>;
}

export const PairedDevicesList: React.FC<PairedDevicesListProps> = memo(({
    devices,
    onOpenPairing,
    onEditDevice,
    onRevokeDevice,
}) => {
    return (
        <div data-testid="paired-devices-container" className="bg-zinc-900/60 backdrop-blur-xl border border-white/5 rounded-2xl p-6 shadow-2xl space-y-5 relative overflow-hidden">
            <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-cyan-500/30 to-transparent" />
            
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
                <div className="flex items-center gap-3">
                    <div className="p-2 rounded-xl bg-cyan-500/10 border border-cyan-500/20 text-cyan-400">
                        <Smartphone className="w-5 h-5" />
                    </div>
                    <div>
                        <h2 className="text-base font-semibold text-zinc-100 tracking-tight flex items-center gap-2">
                            Authorized Mobile Companions
                            <span className="px-2 py-0.5 rounded-full bg-zinc-800 text-[10px] font-mono text-zinc-400">
                                {devices.length}
                            </span>
                        </h2>
                        <p className="text-xs text-zinc-400 mt-0.5">
                            Authenticated mobile companion nodes with cryptographic capability tokens.
                        </p>
                    </div>
                </div>

                <button
                    onClick={onOpenPairing}
                    className="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-cyan-600 to-emerald-600 hover:from-cyan-500 hover:to-emerald-500 text-white text-xs font-semibold rounded-xl shadow-lg shadow-cyan-950/40 transition-all cursor-pointer group active:scale-[0.98]"
                >
                    <QrCode className="w-4 h-4 group-hover:rotate-12 transition-transform" />
                    <span>Display Pairing QR Code</span>
                </button>
            </div>

            {devices.length === 0 ? (
                <div className="p-8 text-center bg-zinc-950/40 border border-zinc-800/60 rounded-xl space-y-2">
                    <p className="text-xs text-zinc-500 italic">No mobile companion devices paired yet.</p>
                    <p className="text-[11px] text-zinc-600">Click the button above to pair a mobile app node via QR code.</p>
                </div>
            ) : (
                <div className="overflow-x-auto">
                    <table className="w-full text-left text-xs border-collapse">
                        <thead>
                            <tr className="border-b border-zinc-800/80 text-[10px] font-mono uppercase tracking-wider text-zinc-500">
                                <th className="pb-3 font-semibold">Device & User</th>
                                <th className="pb-3 font-semibold">Paired Date</th>
                                <th className="pb-3 font-semibold">Key Identifier</th>
                                <th className="pb-3 font-semibold">Status</th>
                                <th className="pb-3 font-semibold text-right">Actions</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-zinc-850">
                            {devices.map(device => (
                                <tr key={device.id} className="group hover:bg-zinc-800/30 transition-colors">
                                    <td className="py-3.5 pr-4">
                                        <div className="font-medium text-zinc-200">{device.name}</div>
                                        <div className="text-[11px] text-zinc-500 font-sans">{device.userName}</div>
                                    </td>
                                    <td className="py-3.5 pr-4 text-zinc-400 font-mono text-[11px]">
                                        {device.pairedAt}
                                    </td>
                                    <td className="py-3.5 pr-4 font-mono text-[11px] text-zinc-400">
                                        <span className="text-zinc-500">Key: </span>
                                        <span className="text-cyan-400/90">{device.key}</span>
                                    </td>
                                    <td className="py-3.5 pr-4">
                                        <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-[10px] font-mono">
                                            <ShieldCheck className="w-3 h-3" />
                                            {device.status}
                                        </span>
                                    </td>
                                    <td className="py-3.5 text-right space-x-1.5">
                                        <Tooltip content="Edit companion device & user alias" position="top">
                                            <button
                                                onClick={() => onEditDevice(device)}
                                                title="Edit Device & User Details"
                                                aria-label="Edit Device & User Details"
                                                className="p-1.5 hover:bg-zinc-800 rounded-lg text-zinc-400 hover:text-zinc-200 transition-colors cursor-pointer"
                                            >
                                                <Edit3 className="w-3.5 h-3.5" />
                                            </button>
                                        </Tooltip>
                                        <Tooltip content="Revoke device access certificate" position="top">
                                            <button
                                                onClick={() => onRevokeDevice(device.id)}
                                                title="Revoke Device Access"
                                                aria-label="Revoke Device Access"
                                                className="p-1.5 hover:bg-red-950/40 rounded-lg text-zinc-500 hover:text-red-400 transition-colors cursor-pointer"
                                            >
                                                <Trash2 className="w-3.5 h-3.5" />
                                            </button>
                                        </Tooltip>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            )}
        </div>
    );
});

PairedDevicesList.displayName = 'PairedDevicesList';

// Metadata: [PairedDevicesList]
