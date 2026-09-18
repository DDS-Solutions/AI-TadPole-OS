/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Agent-Config / DepartmentManagerModal
 * - **Primary Entrypoints**: `DepartmentManagerModal`
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
import { use_department_store } from '../../stores/department_store';
import { event_bus } from '../../services/event_bus';
import { Confirm_Dialog } from '../ui/Confirm_Dialog';

interface DepartmentManagerModalProps {
    isOpen: boolean;
    onClose: () => void;
    currentAgentDept: string;
    onUpdateAgentDept: (newDept: string) => void;
}

export function DepartmentManagerModal({ isOpen, onClose, currentAgentDept, onUpdateAgentDept }: DepartmentManagerModalProps) {
    const { departments, add_department, edit_department, delete_department } = use_department_store();

    const [editingDeptName, setEditingDeptName] = useState<string | null>(null);
    const [editingDeptNewName, setEditingDeptNewName] = useState('');
    const [newDeptName, setNewDeptName] = useState('');

    const [confirmDeleteName, setConfirmDeleteName] = useState<string | null>(null);

    const handleAddNewDept = useCallback(() => {
        const trimmed = newDeptName.trim();
        if (!trimmed) return;
        if (departments.some(d => d.toLowerCase() === trimmed.toLowerCase())) {
            event_bus.emit_log({
                text: 'A department with this name already exists.',
                severity: 'warning',
                source: 'System'
            });
            return;
        }
        add_department(trimmed);
        setNewDeptName('');
        event_bus.emit_log({
            text: `New department created: ${trimmed}`,
            severity: 'success',
            source: 'System'
        });
    }, [newDeptName, departments, add_department]);

    const handleSaveDeptRename = useCallback((oldName: string) => {
        const trimmed = editingDeptNewName.trim();
        if (!trimmed) return;
        edit_department(oldName, trimmed);
        setEditingDeptName(null);
        if (currentAgentDept === oldName) {
            onUpdateAgentDept(trimmed);
        }
        event_bus.emit_log({
            text: `Department renamed to: ${trimmed}`,
            severity: 'info',
            source: 'System'
        });
    }, [editingDeptNewName, currentAgentDept, edit_department, onUpdateAgentDept]);

    const confirmDeleteDept = useCallback(() => {
        if (!confirmDeleteName) return;
        const name = confirmDeleteName;
        setConfirmDeleteName(null);

        delete_department(name);
        event_bus.emit_log({
            text: `Department deleted successfully: ${name}`,
            severity: 'info',
            source: 'System'
        });
    }, [confirmDeleteName, delete_department]);

    const handleDeleteDeptClick = useCallback((name: string) => {
        if (name === currentAgentDept) {
            event_bus.emit_log({
                text: 'Cannot delete the department currently assigned to this agent.',
                severity: 'warning',
                source: 'System'
            });
            return;
        }
        setConfirmDeleteName(name);
    }, [currentAgentDept]);

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
                        <h3 className="text-xs font-bold text-zinc-100 uppercase tracking-[0.2em]">Manage Swarm Departments</h3>
                        <button onClick={onClose} aria-label="Close" className="text-zinc-500 hover:text-zinc-300 p-1.5 rounded hover:bg-zinc-800 cursor-pointer">
                            <X size={16} />
                        </button>
                    </div>
                    <div className="p-5 overflow-y-auto space-y-3 flex-1 custom-scrollbar">
                        {departments.map(d => {
                            const isEditing = editingDeptName === d;
                            return (
                                <div key={d} className="flex items-center justify-between gap-2 p-3 bg-zinc-900/20 rounded-xl border border-zinc-800/40">
                                    {isEditing ? (
                                        <input
                                            value={editingDeptNewName}
                                            onChange={e => setEditingDeptNewName(e.target.value)}
                                            className="bg-zinc-950/45 border border-zinc-700 rounded px-2.5 py-1 text-xs text-zinc-200 focus:outline-none focus:border-green-500/50 flex-1"
                                        />
                                    ) : (
                                        <span className="text-xs text-zinc-300 font-bold uppercase tracking-wider">{d}</span>
                                    )}
                                    <div className="flex items-center gap-1.5 shrink-0">
                                        {isEditing ? (
                                            <>
                                                <button
                                                    onClick={() => handleSaveDeptRename(d)}
                                                    className="p-1 text-green-400 hover:bg-zinc-800 rounded cursor-pointer"
                                                >
                                                    <Check size={14} />
                                                </button>
                                                <button
                                                    onClick={() => setEditingDeptName(null)}
                                                    className="p-1 text-zinc-500 hover:bg-zinc-800 rounded cursor-pointer"
                                                >
                                                    <X size={14} />
                                                </button>
                                            </>
                                        ) : (
                                            <>
                                                <button
                                                    onClick={() => {
                                                        setEditingDeptName(d);
                                                        setEditingDeptNewName(d);
                                                    }}
                                                    className="p-1.5 text-zinc-400 hover:bg-zinc-800 rounded cursor-pointer"
                                                    title="Rename Department"
                                                >
                                                    <Edit2 size={12} />
                                                </button>
                                                <button
                                                    onClick={() => handleDeleteDeptClick(d)}
                                                    className="p-1.5 text-red-400 hover:bg-zinc-800 rounded cursor-pointer"
                                                    title="Delete Department"
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
                            placeholder="New Department Name"
                            value={newDeptName}
                            onChange={e => setNewDeptName(e.target.value)}
                            className="bg-zinc-950/30 border border-zinc-800 rounded-xl px-3.5 py-2 text-xs text-zinc-200 focus:outline-none focus:border-green-500/50 flex-1"
                        />
                        <button
                            onClick={handleAddNewDept}
                            className="px-5 py-2 bg-green-500/10 text-green-500 border border-green-500/20 rounded-xl text-[10px] font-bold uppercase tracking-[0.2em] hover:bg-green-500/20 transition-all cursor-pointer"
                        >
                            Add
                        </button>
                    </div>
                </motion.div>
            </div>

            <Confirm_Dialog
                is_open={confirmDeleteName !== null}
                title="Delete Swarm Department"
                message={`Are you sure you want to delete the department "${confirmDeleteName}"?`}
                on_confirm={confirmDeleteDept}
                on_cancel={() => setConfirmDeleteName(null)}
            />
        </>,
        document.getElementById('portal-root') || document.body
    );
}
