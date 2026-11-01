/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / History / Mission_History_View
 * - **Primary Entrypoints**: `Mission_History_View`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Master-detail layout syncing state with URL search params (`?id=...`).
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: `Failed to load mission catalog`, `Mission dossier could not be loaded`, `Failed to toggle pin`
 * - **Telemetry Targets**: none declared
 */

import React, { useState, useEffect, useCallback, useRef } from 'react';
import { useSearchParams } from 'react-router-dom';
import { Layers, AlertCircle, RefreshCw } from 'lucide-react';
import { mission_history_service } from '../../../services/mission_history_service';
import type { MissionSummaryItem, MissionDossier } from '../../../types/mission_dossier';
import { use_notification_store } from '../../../stores/notification_store';
import { useWebSocketEvent } from '../../../hooks/use_web_socket_event';
import { Mission_Catalog_Sidebar } from './Mission_Catalog_Sidebar';
import { Mission_Flight_Dossier } from './Mission_Flight_Dossier';

export const Mission_History_View: React.FC = () => {
    const [search_params, set_search_params] = useSearchParams();
    const url_mission_id = search_params.get('id');
    const add_notification = use_notification_store((s) => s.add_notification);

    const [missions, set_missions] = useState<MissionSummaryItem[]>([]);
    const [total, set_total] = useState(0);
    const [page, set_page] = useState(1);
    const [status_filter, set_status_filter] = useState<string>('all');
    const [search_query, set_search_query] = useState('');
    const [is_catalog_loading, set_is_catalog_loading] = useState(true);
    const [catalog_error, set_catalog_error] = useState<string | null>(null);

    const [selected_mission_id, set_selected_mission_id] = useState<string | null>(url_mission_id);
    const [dossier, set_dossier] = useState<MissionDossier | null>(null);
    const [is_dossier_loading, set_is_dossier_loading] = useState(false);
    const [dossier_error, set_dossier_error] = useState<string | null>(null);
    const [is_live_syncing, set_is_live_syncing] = useState(false);
    const sync_timer_ref = useRef<ReturnType<typeof setTimeout> | null>(null);
    const ws_debounce_timer_ref = useRef<ReturnType<typeof setTimeout> | null>(null);
    const latest_catalog_req_id = useRef<number>(0);
    const latest_dossier_req_id = useRef<number>(0);
    const initial_auto_selected = useRef<boolean>(false);
    const url_mission_id_ref = useRef<string | null>(url_mission_id);
    const selected_mission_id_ref = useRef<string | null>(selected_mission_id);



    // 1. Fetch catalog with request ordering protection (S-07)
    const load_catalog = useCallback(async () => {
        const req_id = ++latest_catalog_req_id.current;
        set_is_catalog_loading(true);
        set_catalog_error(null);
        try {
            const res = await mission_history_service.fetch_catalog({
                page,
                per_page: 25,
                status: status_filter,
                search: search_query.trim() || undefined,
            });
            if (req_id !== latest_catalog_req_id.current) return;
            if (res.success) {
                set_missions(res.data);
                set_total(res.total);

                // Auto-select first mission ONLY once on initial mount if none selected in URL or state
                if (!initial_auto_selected.current && !url_mission_id_ref.current && !selected_mission_id_ref.current && res.data.length > 0) {
                    initial_auto_selected.current = true;
                    const first_id = res.data[0].id;
                    set_selected_mission_id(first_id);
                    set_search_params((prev) => {
                        const next = new URLSearchParams(prev);
                        next.set('id', first_id);
                        return next;
                    });
                }
            } else {
                set_catalog_error((res as { error?: string }).error || 'Failed to load mission catalog');
            }
        } catch (err: unknown) {
            if (req_id !== latest_catalog_req_id.current) return;
            const msg = err instanceof Error ? err.message : 'Failed to connect to backend engine';
            console.error('Failed to load mission catalog:', err);
            set_catalog_error(msg);
        } finally {
            if (req_id === latest_catalog_req_id.current) {
                set_is_catalog_loading(false);
            }
        }
    }, [page, status_filter, search_query, set_search_params]);

    useEffect(() => {
        // eslint-disable-next-line react-hooks/set-state-in-effect
        void load_catalog();
    }, [load_catalog]);

    // 2. Fetch specific dossier when selected_mission_id changes with request ordering protection (S-07)
    const load_dossier = useCallback(async (id: string) => {
        const req_id = ++latest_dossier_req_id.current;
        set_is_dossier_loading(true);
        set_dossier_error(null);
        try {
            const data = await mission_history_service.fetch_dossier(id);
            if (req_id !== latest_dossier_req_id.current) return;
            set_dossier(data);
        } catch (err: unknown) {
            if (req_id !== latest_dossier_req_id.current) return;
            console.error(`Failed to load dossier for mission ${id}:`, err);
            set_dossier_error(err instanceof Error ? err.message : 'Mission dossier could not be loaded');
        } finally {
            if (req_id === latest_dossier_req_id.current) {
                set_is_dossier_loading(false);
            }
        }
    }, []);

    useEffect(() => {
        if (selected_mission_id) {
            // eslint-disable-next-line react-hooks/set-state-in-effect
            void load_dossier(selected_mission_id);
        } else {
            latest_dossier_req_id.current++;
            set_dossier(null);
            set_is_dossier_loading(false);
            set_dossier_error(null);
        }
    }, [selected_mission_id, load_dossier]);

    // Hybrid Streaming: Real-time WebSocket notifications with debounced authoritative REST pulls (S-08)
    const handle_stream_event = useCallback((data: unknown) => {
        const evt = data as { type?: string; mission_id?: string; category?: string; tags?: string[] };
        const is_mission_event =
            evt?.type === 'mission_update' ||
            evt?.category === 'mission' ||
            (Array.isArray(evt?.tags) && evt.tags.includes('mission')) ||
            (evt?.type === 'log' && evt?.category === 'mission');

        if (is_mission_event) {
            set_is_live_syncing(true);
            if (sync_timer_ref.current) clearTimeout(sync_timer_ref.current);
            sync_timer_ref.current = setTimeout(() => {
                set_is_live_syncing(false);
            }, 1200);

            // True debounced authoritative state sync (S-08: 800ms coalesce)
            if (ws_debounce_timer_ref.current) clearTimeout(ws_debounce_timer_ref.current);
            ws_debounce_timer_ref.current = setTimeout(() => {
                void load_catalog();
                if (selected_mission_id && (!evt.mission_id || evt.mission_id === selected_mission_id)) {
                    void load_dossier(selected_mission_id);
                }
            }, 800);
        }
    }, [load_catalog, load_dossier, selected_mission_id]);

    useWebSocketEvent('raw', handle_stream_event, 500);

    // Clean up timers on unmount
    useEffect(() => {
        return () => {
            if (sync_timer_ref.current) clearTimeout(sync_timer_ref.current);
            if (ws_debounce_timer_ref.current) clearTimeout(ws_debounce_timer_ref.current);
        };
    }, []);

    // Sync state if URL changes externally (R-05: Handles back-navigation clearing ?id=)
    useEffect(() => {
        if (url_mission_id) {
            if (url_mission_id !== selected_mission_id) {
                // eslint-disable-next-line react-hooks/set-state-in-effect
                set_selected_mission_id(url_mission_id);
            }
        } else if (selected_mission_id !== null) {
            set_selected_mission_id(null);
            set_dossier(null);
        }
    }, [url_mission_id, selected_mission_id]);

    const handle_select_mission = (id: string) => {
        selected_mission_id_ref.current = id;
        set_selected_mission_id(id);
        set_search_params((prev) => {
            const next = new URLSearchParams(prev);
            next.set('id', id);
            return next;
        });
    };

    const handle_toggle_pin = async (id: string) => {
        try {
            const res = await mission_history_service.toggle_pin(id);
            if (res.success) {
                // Update in catalog
                set_missions((prev) =>
                    prev.map((m) => (m.id === id ? { ...m, is_pinned: res.is_pinned } : m))
                );
                // Update in current dossier if open
                if (dossier && dossier.mission.id === id) {
                    set_dossier({
                        ...dossier,
                        mission: { ...dossier.mission, is_pinned: res.is_pinned },
                    });
                }
                add_notification({
                    title: res.is_pinned ? 'Mission Pinned' : 'Mission Unpinned',
                    message: `Mission ${id.slice(0, 8)} retention lock has been ${res.is_pinned ? 'enabled' : 'removed'}.`,
                    severity: 'info',
                    persistent: false,
                });
            } else {
                add_notification({
                    title: 'Pin Update Failed',
                    message: (res as { error?: string }).error || 'Backend rejected mission pin update.',
                    severity: 'error',
                    persistent: false,
                });
            }
        } catch (err: unknown) {
            const msg = err instanceof Error ? err.message : 'Failed to toggle retention status';
            console.error(`Failed to toggle pin for ${id}:`, err);
            add_notification({
                title: 'Pin Update Failed',
                message: msg,
                severity: 'error',
                persistent: false,
            });
        }
    };

    return (
        <div className="flex h-full w-full overflow-hidden bg-zinc-950">
            {/* Left Sidebar Catalog */}
            <Mission_Catalog_Sidebar
                missions={missions}
                selected_id={selected_mission_id}
                is_loading={is_catalog_loading}
                error={catalog_error}
                total={total}
                page={page}
                per_page={25}
                search_query={search_query}
                status_filter={status_filter}
                on_select_mission={handle_select_mission}
                on_search_change={(q) => {
                    set_search_query(q);
                    set_page(1);
                }}
                on_status_change={(s) => {
                    set_status_filter(s);
                    set_page(1);
                }}
                on_page_change={set_page}
                on_toggle_pin={handle_toggle_pin}
                on_refresh={load_catalog}
            />

            {/* Right Master Dossier Pane */}
            <div className="flex-1 flex flex-col h-full overflow-hidden">
                {is_dossier_loading ? (
                    <div className="flex-1 flex flex-col items-center justify-center p-12 text-zinc-400 gap-3">
                        <RefreshCw size={28} className="animate-spin text-cyan-400" />
                        <span className="text-xs font-mono">Compiling Flight Recorder Dossier...</span>
                    </div>
                ) : dossier_error ? (
                    <div className="flex-1 flex flex-col items-center justify-center p-12 text-center text-rose-400 gap-3">
                        <AlertCircle size={32} />
                        <span className="text-sm font-semibold">{dossier_error}</span>
                        <button
                            onClick={() => selected_mission_id && load_dossier(selected_mission_id)}
                            className="px-3 py-1.5 rounded-lg bg-zinc-900 border border-zinc-800 text-xs text-zinc-300 hover:text-zinc-100"
                        >
                            Retry Loading Dossier
                        </button>
                    </div>
                ) : dossier ? (
                    <Mission_Flight_Dossier
                        key={dossier.mission.id}
                        dossier={dossier}
                        on_toggle_pin={handle_toggle_pin}
                        is_live_syncing={is_live_syncing}
                    />
                ) : (
                    <div className="flex-1 flex flex-col items-center justify-center p-12 text-center text-zinc-500">
                        <Layers size={36} className="text-zinc-600 mb-2" />
                        <span className="text-sm font-semibold text-zinc-400">Select a mission</span>
                        <span className="text-xs text-zinc-600 mt-1">
                            Choose a mission from the catalog sidebar to inspect its flight records
                        </span>
                    </div>
                )}
            </div>
        </div>
    );
};
