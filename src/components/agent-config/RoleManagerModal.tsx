/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Agent-Config / RoleManagerModal
 * - **Primary Entrypoints**: `RoleManagerModal`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { useState, useCallback } from 'react';
import { createPortal } from 'react-dom';
import { X, Check, Edit2, Trash2 } from 'lucide-react';
import { motion } from 'framer-motion';
import { use_role_store } from '../../stores/role_store';
import { tadpole_os_service } from '../../services/tadpoleos_service';
import { event_bus } from '../../services/event_bus';
import { ValidationUtils } from '../../utils/validation_utils';
import { Confirm_Dialog } from '../ui/Confirm_Dialog';

interface RoleManagerModalProps {
    isOpen: boolean;
    onClose: () => void;
    currentAgentRole: string;
}

export function RoleManagerModal({ isOpen, onClose, currentAgentRole }: RoleManagerModalProps) {
    const roles = use_role_store(s => s.roles);
    const add_role = use_role_store(s => s.add_role);
    const update_role = use_role_store(s => s.update_role);
    const delete_role = use_role_store(s => s.delete_role);

    const [editingRoleId, setEditingRoleId] = useState<string | null>(null);
    const [editingRoleName, setEditingRoleName] = useState('');
    const [newRoleName, setNewRoleName] = useState('');

    const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null);

    const availableRoles = Object.keys(roles);

    const handleAddNewRole = useCallback(async () => {
        const trimmed = newRoleName.trim();
        if (!trimmed) return;

        const r_id = ValidationUtils.sanitize_id(trimmed);
        if (roles[r_id]) {
            event_bus.emit_log({
                text: 'A role with this name/ID already exists.',
                severity: 'warning',
                source: 'System'
            });
            return;
        }

        const new_role = {
            id: r_id,
            name: trimmed,
            department: 'Operations' as const,
            description: `Custom ${trimmed} role.`,
            skills: [],
            workflows: [],
            mcp_tools: [],
            requires_oversight: false,
            created_at: new Date().toISOString()
        };

        try {
            await tadpole_os_service.save_role_blueprint(new_role);
        } catch (err) {
            console.error('Failed to save new role blueprint:', err);
        }
        add_role(new_role);
        setNewRoleName('');
        event_bus.emit_log({
            text: `New role created: ${trimmed}`,
            severity: 'success',
            source: 'System'
        });
    }, [newRoleName, roles, add_role]);

    const handleSaveRoleRename = useCallback(async (id: string) => {
        const trimmed = editingRoleName.trim();
        if (!trimmed) return;
        const existing = roles[id];
        if (existing) {
            const updated = { ...existing, name: trimmed };
            try {
                await tadpole_os_service.save_role_blueprint(updated);
            } catch (err) {
                console.error('Failed to rename role:', err);
            }
            update_role(id, { name: trimmed });
            setEditingRoleId(null);
            event_bus.emit_log({
                text: `Role renamed to: ${trimmed}`,
                severity: 'info',
                source: 'System'
            });
        }
    }, [editingRoleName, roles, update_role]);

    const confirmDeleteRole = useCallback(async () => {
        if (!confirmDeleteId) return;
        const id = confirmDeleteId;
        setConfirmDeleteId(null);

        try {
            await tadpole_os_service.delete_role_blueprint(id);
        } catch (err) {
            console.error('Failed to delete role blueprint:', err);
        }
        delete_role(id);
        event_bus.emit_log({
            text: `Role template deleted successfully: ${id}`,
            severity: 'info',
            source: 'System'
        });
    }, [confirmDeleteId, delete_role]);

    const handleDeleteRoleClick = useCallback((id: string) => {
        if (id === currentAgentRole) {
            event_bus.emit_log({
                text: 'Cannot delete the role currently assigned to this agent.',
                severity: 'warning',
                source: 'System'
            });
            return;
        }
        setConfirmDeleteId(id);
    }, [currentAgentRole]);

    if (!isOpen) return null;

    return createPortal(
        <>
            <div className="fixed inset-0 z-[200] flex items-center justify-center p-4 bg-zinc-950/80 backdrop-blur-sm pointer-events-auto" onClick={onClose}>
                <motion.div
                    initial={{ opacity: 0, scale: 0.95 }}
                    animate={{ opacity: 1, scale: 1 }}
                    exit={{ opacity: 0, scale: 0.95 }}
                    className="bg-[#18181b] border border-zinc-800 rounded-3xl w-full max-w-md shadow-2xl overflow-hidden flex flex-col max-h-[70vh]"
                    onClick={e => e.stopPropagation()}
                >
                    <div className="p-5 border-b border-zinc-800 flex items-center justify-between bg-zinc-900/40">
                        <h3 className="text-xs font-bold text-zinc-100 uppercase tracking-[0.2em]">Manage Swarm Roles</h3>
                        <button onClick={onClose} aria-label="Close" className="text-zinc-500 hover:text-zinc-300 p-1.5 rounded hover:bg-zinc-800 cursor-pointer">
                            <X size={16} />
                        </button>
                    </div>
                    <div className="p-5 overflow-y-auto space-y-3 flex-1 custom-scrollbar">
                        {availableRoles.map(r => {
                            const roleObj = roles[r];
                            const roleName = roleObj?.name || r;
                            const isEditing = editingRoleId === r;
                            return (
                                <div key={r} className="flex items-center justify-between gap-2 p-3 bg-zinc-900/20 rounded-xl border border-zinc-800/40">
                                    {isEditing ? (
                                        <input
                                            value={editingRoleName}
                                            onChange={e => setEditingRoleName(e.target.value)}
                                            className="bg-zinc-950/45 border border-zinc-700 rounded px-2.5 py-1 text-xs text-zinc-200 focus:outline-none focus:border-emerald-500/50 flex-1"
                                        />
                                    ) : (
                                        <span className="text-xs text-zinc-300 font-bold uppercase tracking-wider">{roleName}</span>
                                    )}
                                    <div className="flex items-center gap-1.5 shrink-0">
                                        {isEditing ? (
                                            <>
                                                <button
                                                    onClick={() => handleSaveRoleRename(r)}
                                                    className="p-1 text-green-400 hover:bg-zinc-800 rounded cursor-pointer"
                                                >
                                                    <Check size={14} />
                                                </button>
                                                <button
                                                    onClick={() => setEditingRoleId(null)}
                                                    className="p-1 text-zinc-500 hover:bg-zinc-800 rounded cursor-pointer"
                                                >
                                                    <X size={14} />
                                                </button>
                                            </>
                                        ) : (
                                            <>
                                                <button
                                                    onClick={() => {
                                                        setEditingRoleId(r);
                                                        setEditingRoleName(roleName);
                                                    }}
                                                    className="p-1.5 text-zinc-400 hover:bg-zinc-800 rounded cursor-pointer"
                                                    title="Rename Role"
                                                >
                                                    <Edit2 size={12} />
                                                </button>
                                                <button
                                                    onClick={() => handleDeleteRoleClick(r)}
                                                    className="p-1.5 text-red-400 hover:bg-zinc-800 rounded cursor-pointer"
                                                    title="Delete Role"
                                                >
                                                    <Trash2 size={12} />
                                                </button>
                                            </>
                                        )}
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                    <div className="p-5 border-t border-zinc-800 bg-zinc-900/30 flex gap-2">
                        <input
                            placeholder="New Role Name"
                            value={newRoleName}
                            onChange={e => setNewRoleName(e.target.value)}
                            className="bg-zinc-950/30 border border-zinc-800 rounded-xl px-3.5 py-2 text-xs text-zinc-200 focus:outline-none focus:border-emerald-500/50 flex-1"
                        />
                        <button
                            onClick={handleAddNewRole}
                            className="px-5 py-2 bg-emerald-500/10 text-emerald-500 border border-emerald-500/20 rounded-xl text-[10px] font-bold uppercase tracking-[0.2em] hover:bg-emerald-500/20 transition-all cursor-pointer"
                        >
                            Add
                        </button>
                    </div>
                </motion.div>
            </div>

            <Confirm_Dialog
                is_open={confirmDeleteId !== null}
                title="Delete Swarm Role"
                message={`Are you sure you want to delete the role "${roles[confirmDeleteId || '']?.name || confirmDeleteId}"?`}
                on_confirm={confirmDeleteRole}
                on_cancel={() => setConfirmDeleteId(null)}
            />
        </>,
        document.getElementById('portal-root') || document.body
    );
}
