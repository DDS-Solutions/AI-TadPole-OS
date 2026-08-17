/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Companion Device Pairing & Edit Modal Component**
 * Renders multi-step Zero-Trust QR pairing flow, cryptographic challenge display,
 * and device alias modification dialog adhering to design.md.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Malformed QR payload string or unhandled modal step transition.
 * - **Telemetry Link**: Search `[DevicePairingModal]` in UI logs.
 */

import React, { memo } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { X, QrCode, Smartphone, User, CheckCircle2 } from 'lucide-react';
import type { CompanionDevice } from './PairedDevicesList';

export interface DevicePairingModalProps {
    pairingStep: 'closed' | 'config' | 'qr';
    pairingToken: string;
    qrTargetIp: string;
    deviceNameInput: string;
    userNameInput: string;
    onDeviceNameChange: (val: string) => void;
    onUserNameChange: (val: string) => void;
    onClosePairing: () => void;
    onProceedToQr: () => void;
    onBackToConfig: () => void;
    onCompletePairing: () => void | Promise<void>;

    // Edit modal
    editingDevice: CompanionDevice | null;
    editNameInput: string;
    editUserInput: string;
    onEditNameChange: (val: string) => void;
    onEditUserChange: (val: string) => void;
    onCloseEdit: () => void;
    onSaveEdit: () => void | Promise<void>;
}

export const DevicePairingModal: React.FC<DevicePairingModalProps> = memo(({
    pairingStep,
    pairingToken,
    qrTargetIp,
    deviceNameInput,
    userNameInput,
    onDeviceNameChange,
    onUserNameChange,
    onClosePairing,
    onProceedToQr,
    onBackToConfig,
    onCompletePairing,
    editingDevice,
    editNameInput,
    editUserInput,
    onEditNameChange,
    onEditUserChange,
    onCloseEdit,
    onSaveEdit,
}) => {
    return (
        <>
            {/* Step 1: Device Configuration Dialog */}
            {pairingStep === 'config' && (
                <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm animate-in fade-in duration-200">
                    <div className="bg-zinc-900 border border-zinc-800 rounded-2xl w-full max-w-md p-6 shadow-2xl space-y-6 relative overflow-hidden">
                        <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-cyan-500/40 to-transparent" />
                        
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-2.5">
                                <div className="p-2 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/20">
                                    <Smartphone className="w-5 h-5" />
                                </div>
                                <h3 className="text-base font-semibold text-zinc-100">
                                    Configure Companion Device Details
                                </h3>
                            </div>
                            <button
                                onClick={onClosePairing}
                                className="p-1.5 hover:bg-zinc-800 rounded-lg text-zinc-400 hover:text-zinc-200 transition-colors cursor-pointer"
                                aria-label="Close dialog"
                            >
                                <X className="w-4 h-4" />
                            </button>
                        </div>

                        <div className="space-y-4 text-xs">
                            <div>
                                <label htmlFor="user-name-input" className="block text-zinc-400 font-medium mb-1.5 uppercase tracking-wider text-[10px] font-mono">
                                    User / Operator Name
                                </label>
                                <div className="relative">
                                    <User className="w-4 h-4 text-zinc-500 absolute left-3 top-3" />
                                    <input
                                        id="user-name-input"
                                        aria-label="User / Operator Name"
                                        type="text"
                                        value={userNameInput}
                                        onChange={(e) => onUserNameChange(e.target.value)}
                                        placeholder="e.g. Sovereign Operator"
                                        className="w-full pl-9 pr-3 py-2.5 bg-zinc-950/60 border border-zinc-800 rounded-xl text-zinc-100 placeholder-zinc-500 focus:outline-none focus:border-cyan-500/60 focus:ring-1 focus:ring-cyan-500/30 transition-all font-sans"
                                    />
                                </div>
                            </div>

                            <div>
                                <label htmlFor="device-name-input" className="block text-zinc-400 font-medium mb-1.5 uppercase tracking-wider text-[10px] font-mono">
                                    Companion Device Name
                                </label>
                                <div className="relative">
                                    <Smartphone className="w-4 h-4 text-zinc-500 absolute left-3 top-3" />
                                    <input
                                        id="device-name-input"
                                        aria-label="Companion Device Name"
                                        type="text"
                                        value={deviceNameInput}
                                        onChange={(e) => onDeviceNameChange(e.target.value)}
                                        placeholder="e.g. Android Companion (Pixel 8)"
                                        className="w-full pl-9 pr-3 py-2.5 bg-zinc-950/60 border border-zinc-800 rounded-xl text-zinc-100 placeholder-zinc-500 focus:outline-none focus:border-cyan-500/60 focus:ring-1 focus:ring-cyan-500/30 transition-all font-sans"
                                    />
                                </div>
                            </div>
                        </div>

                        <div className="flex items-center justify-end gap-3 pt-2">
                            <button
                                onClick={onClosePairing}
                                className="px-4 py-2 text-xs text-zinc-400 hover:text-zinc-200 bg-zinc-800/60 hover:bg-zinc-800 border border-zinc-700/60 rounded-xl transition-all cursor-pointer"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={onProceedToQr}
                                className="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-cyan-600 to-emerald-600 hover:from-cyan-500 hover:to-emerald-500 text-white text-xs font-semibold rounded-xl shadow-lg shadow-cyan-950/40 transition-all cursor-pointer active:scale-[0.98]"
                            >
                                <QrCode className="w-4 h-4" />
                                <span>Generate QR Code</span>
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Step 2: QR Code Pairing Screen */}
            {pairingStep === 'qr' && (
                <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm animate-in fade-in duration-200">
                    <div className="bg-zinc-900 border border-zinc-800 rounded-2xl w-full max-w-md p-6 shadow-2xl space-y-6 relative overflow-hidden text-center">
                        <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-emerald-500/40 to-transparent" />
                        
                        <div className="flex items-center justify-between text-left">
                            <div className="flex items-center gap-2.5">
                                <div className="p-2 rounded-xl bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                                    <QrCode className="w-5 h-5" />
                                </div>
                                <div>
                                    <h3 className="text-base font-semibold text-zinc-100">
                                        Scan QR Code to Complete Pairing
                                    </h3>
                                    <p className="text-[11px] text-zinc-400">
                                        {deviceNameInput || 'Android Companion Device'} ({userNameInput || 'Sovereign Operator'})
                                    </p>
                                </div>
                            </div>
                            <button
                                onClick={onClosePairing}
                                className="p-1.5 hover:bg-zinc-800 rounded-lg text-zinc-400 hover:text-zinc-200 transition-colors cursor-pointer"
                                aria-label="Close dialog"
                            >
                                <X className="w-4 h-4" />
                            </button>
                        </div>

                        {/* QR Code Container */}
                        <div className="p-4 bg-white rounded-2xl inline-block shadow-xl border border-zinc-200">
                            <QRCodeSVG
                                value={`tadpole://pair?ip=${qrTargetIp}&token=${pairingToken}&user=${encodeURIComponent(userNameInput || 'Sovereign Operator')}&device=${encodeURIComponent(deviceNameInput || 'Android Companion Device')}`}
                                size={190}
                                level="M"
                                includeMargin={false}
                            />
                        </div>

                        <div className="space-y-2">
                            <div className="p-2.5 bg-zinc-950/60 border border-zinc-800 rounded-xl text-xs font-mono text-zinc-300 flex items-center justify-center gap-2">
                                <span className="text-zinc-500 font-sans">Pairing Challenge Code:</span>
                                <span className="font-bold text-emerald-400">{pairingToken}</span>
                            </div>
                            <p className="text-[11px] text-zinc-500">
                                Target Endpoint: <span className="font-mono text-zinc-400">{qrTargetIp}</span>
                            </p>
                        </div>

                        <div className="space-y-2 pt-2">
                            <button
                                onClick={onCompletePairing}
                                className="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-semibold rounded-xl shadow-lg shadow-emerald-950/40 transition-all cursor-pointer active:scale-[0.98]"
                            >
                                <CheckCircle2 className="w-4 h-4" />
                                <span>Refresh Pairing Status</span>
                            </button>
                            <div className="flex items-center justify-between gap-2 pt-1">
                                <button
                                    onClick={onBackToConfig}
                                    className="px-3 py-1.5 text-xs text-zinc-400 hover:text-zinc-200 transition-colors cursor-pointer"
                                >
                                    Back to Details
                                </button>
                                <button
                                    onClick={onClosePairing}
                                    className="px-3 py-1.5 text-xs text-zinc-400 hover:text-zinc-200 transition-colors cursor-pointer"
                                >
                                    Done / Close
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            )}

            {/* Edit Device Modal */}
            {editingDevice && (
                <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm animate-in fade-in duration-200">
                    <div className="bg-zinc-900 border border-zinc-800 rounded-2xl w-full max-w-md p-6 shadow-2xl space-y-6 relative overflow-hidden">
                        <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-cyan-500/40 to-transparent" />
                        
                        <div className="flex items-center justify-between">
                            <h3 className="text-base font-semibold text-zinc-100">
                                Edit Authorized Paired Device
                            </h3>
                            <button
                                onClick={onCloseEdit}
                                className="p-1.5 hover:bg-zinc-800 rounded-lg text-zinc-400 hover:text-zinc-200 transition-colors cursor-pointer"
                                aria-label="Close dialog"
                            >
                                <X className="w-4 h-4" />
                            </button>
                        </div>

                        <div className="space-y-4 text-xs">
                            <div>
                                <label htmlFor="edit-user-input" className="block text-zinc-400 font-medium mb-1.5 uppercase tracking-wider text-[10px] font-mono">
                                    Edit User Name
                                </label>
                                <input
                                    id="edit-user-input"
                                    aria-label="Edit User Name"
                                    type="text"
                                    value={editUserInput}
                                    onChange={(e) => onEditUserChange(e.target.value)}
                                    className="w-full px-3 py-2.5 bg-zinc-950/60 border border-zinc-800 rounded-xl text-zinc-100 focus:outline-none focus:border-cyan-500/60 focus:ring-1 focus:ring-cyan-500/30 transition-all font-sans"
                                />
                            </div>

                            <div>
                                <label htmlFor="edit-device-input" className="block text-zinc-400 font-medium mb-1.5 uppercase tracking-wider text-[10px] font-mono">
                                    Edit Device Name
                                </label>
                                <input
                                    id="edit-device-input"
                                    aria-label="Edit Device Name"
                                    type="text"
                                    value={editNameInput}
                                    onChange={(e) => onEditNameChange(e.target.value)}
                                    className="w-full px-3 py-2.5 bg-zinc-950/60 border border-zinc-800 rounded-xl text-zinc-100 focus:outline-none focus:border-cyan-500/60 focus:ring-1 focus:ring-cyan-500/30 transition-all font-sans"
                                />
                            </div>
                        </div>

                        <div className="flex items-center justify-end gap-3 pt-2">
                            <button
                                onClick={onCloseEdit}
                                className="px-4 py-2 text-xs text-zinc-400 hover:text-zinc-200 bg-zinc-800/60 hover:bg-zinc-800 border border-zinc-700/60 rounded-xl transition-all cursor-pointer"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={onSaveEdit}
                                className="px-4 py-2 bg-cyan-600 hover:bg-cyan-500 text-white text-xs font-semibold rounded-xl shadow-lg shadow-cyan-950/40 transition-all cursor-pointer active:scale-[0.98]"
                            >
                                Save Changes
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </>
    );
});

DevicePairingModal.displayName = 'DevicePairingModal';

// Metadata: [DevicePairingModal]
