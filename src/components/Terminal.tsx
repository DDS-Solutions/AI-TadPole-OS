/**
 * @docs ARCHITECTURE:Interface
 * @docs OPERATIONS_MANUAL:Navigation
 * 
 * ### AI Assist Note
 * **UI Component**: Dashboard-fixed CLI portal for system logs and slash-commands. 
 * Orchestrates event bus ingestion, log buffering (500 max), and intelligent tab-autocomplete for swarm commands.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Log buffer overflow under high-frequency event bursts, command processing timeout (30s), or autocomplete UI flicker during rapid typing.
 * - **Telemetry Link**: Search for `[Terminal]` or `process_command` in UI logs.
 */

/**
 * @module Terminal
 * Collapsible CLI-style interface fixed to the bottom of the dashboard.
 * Subscribes to the shared {@link event_bus} for real-time log display,
 * delegates slash-commands to the {@link process_command} service,
 * and features intelligent tab-autocomplete for commands and agents.
 * Refactored for strict snake_case compliance for backend parity.
 */
import React, { useState, useEffect, useRef } from 'react';
import { Terminal as Terminal_Icon } from 'lucide-react';
import { event_bus } from '../services/event_bus';
import type { log_entry } from '../services/event_bus';
import { use_settings_store } from '../stores/settings_store';
import { process_command } from '../logic/command_processor';
import type { Agent } from '../types';
import { Tooltip } from './ui';
import { i18n } from '../i18n';
import { system_api_service } from '../services/system_api_service';

/** Maximum number of log entries to retain in the Terminal buffer. */
const MAX_LOG_ENTRIES = 500;

/** Static list of available CLI commands. */
const AVAILABLE_COMMANDS = ['/help', '/clear', '/pause', '/resume', '/kill', '/status', '/swarm status', '/swarm optimize'];

interface Terminal_Props {
    /** Current agent list, passed to the command processor for lookups. */
    agents: Agent[];
}

/**
 * Terminal_Component
 * Collapsible terminal panel.
 * - Displays a scrollable log of all {@link log_entry} events from the event_bus.
 * - Provides a `> _` command input for issuing slash-commands.
 * - Renders fixed at the bottom of the viewport, above the sidebar.
 */
/**
 * Log_Entry_Item
 * Memoized sub-component to prevent expensive re-renders and string formatting
 * (like toLocaleTimeString) for every log entry in the buffer.
 */
const Log_Entry_Item = React.memo(({ log }: { log: log_entry }) => {
    // Memoize the timestamp string to avoid calling toLocaleTimeString on every render cycle
    const timestamp_str = React.useMemo(() => log.timestamp.toLocaleTimeString(), [log.timestamp]);

    // Truncate extremely large payloads to prevent layout engine collapse (max 10k chars for terminal display)
    const display_text = React.useMemo(() => {
        if (log.text && log.text.length > 10000) {
            return log.text.substring(0, 10000) + '... [TRUNCATED]';
        }
        return log.text;
    }, [log.text]);

    return (
        <div className="flex gap-2">
            <span className="text-zinc-600 shrink-0">[{timestamp_str}]</span>
            <span className={`font-bold shrink-0 ${log.source === 'User' ? 'text-green-400' :
                log.source === 'Agent' ? 'text-green-400' :
                    'text-zinc-400'
                }`}>
                {log.source === 'Agent' && log.agent_id ? `${log.agent_id}:` : `${log.source}:`}
            </span>
            <span className={`break-words whitespace-pre-wrap ${log.severity === 'error' ? 'text-red-400' :
                log.severity === 'warning' ? 'text-yellow-400' :
                    'text-zinc-300'
                }`}>{display_text}</span>
        </div>
    );
});

/**
 * Terminal_Component
 * Collapsible terminal panel.
 * - Displays a scrollable log of all {@link log_entry} events from the event_bus.
 * - Provides a `> _` command input for issuing slash-commands.
 * - Renders fixed at the bottom of the viewport, above the sidebar.
 */
export default function Terminal_Component({ agents }: Terminal_Props) {
    const [is_open, set_is_open] = useState(false); // Collapsible
    const [input_value, set_input_value] = useState('');
    const [render_version, set_render_version] = useState(0); // For triggering re-renders on buffer updates
    const log_end_ref = useRef<HTMLDivElement>(null);
    const log_container_ref = useRef<HTMLDivElement>(null);
    const { settings } = use_settings_store();
    const is_safe_mode = settings.is_safe_mode;

    const input_ref = useRef<HTMLInputElement>(null);
    const [workspace_files, set_workspace_files] = useState<string[]>([]);
    const [file_suggestions, set_file_suggestions] = useState<string[]>([]);
    const [file_suggestion_index, set_file_suggestion_index] = useState(-1);
    const [mention_start_index, set_mention_start_index] = useState(-1);

    const load_workspace_files = async () => {
        try {
            const files = await system_api_service.get_workspace_files();
            set_workspace_files(files);
        } catch (err) {
            console.error('[Terminal] Failed to fetch workspace files:', err);
        }
    };

    const toggle_open = () => {
        const next_open = !is_open;
        set_is_open(next_open);
        if (next_open) {
            load_workspace_files();
        } else {
            set_file_suggestions([]);
            set_file_suggestion_index(-1);
            set_mention_start_index(-1);
        }
    };

    const check_mention = React.useCallback((value: string, selection_start: number) => {
        const text_before_cursor = value.slice(0, selection_start);
        const match = text_before_cursor.match(/@(\S*)$/);
        
        if (match) {
            const query = match[1].toLowerCase();
            const start_idx = text_before_cursor.lastIndexOf('@');
            set_mention_start_index(start_idx);
            
            const filtered = workspace_files.filter(f => f.toLowerCase().includes(query));
            set_file_suggestions(filtered.slice(0, 10));
            set_file_suggestion_index(prev => {
                if (filtered.length === 0) return -1;
                if (prev >= filtered.length || prev === -1) return 0;
                return prev;
            });
        } else {
            set_file_suggestions([]);
            set_file_suggestion_index(-1);
            set_mention_start_index(-1);
        }
    }, [workspace_files]);

    useEffect(() => {
        if (is_open && workspace_files.length > 0 && input_value) {
            check_mention(input_value, input_ref.current?.selectionStart ?? 0);
        }
    }, [workspace_files, is_open, input_value, check_mention]);

    const complete_mention = (file_path: string) => {
        if (mention_start_index === -1) return;
        
        const cursor_pos = input_ref.current?.selectionStart || 0;
        const before = input_value.slice(0, mention_start_index);
        const after = input_value.slice(cursor_pos);
        const completed_val = `${before}${file_path} ${after}`;
        
        set_input_value(completed_val);
        set_file_suggestions([]);
        set_file_suggestion_index(-1);
        set_mention_start_index(-1);
        
        setTimeout(() => {
            if (input_ref.current) {
                const new_cursor_pos = before.length + file_path.length + 1;
                input_ref.current.focus();
                input_ref.current.setSelectionRange(new_cursor_pos, new_cursor_pos);
            }
        }, 0);
    };

    /** Persistent ring buffer to minimize GC pressure during high-frequency logging. */
    const internal_buffer_ref = useRef<(log_entry | null)[]>(new Array(MAX_LOG_ENTRIES).fill(null));
    const head_ref = useRef(0); // Points to the next write position

    /** Incoming event batch, flushed to the ring buffer on each animation frame. */
    const update_batch_ref = useRef<log_entry[]>([]);
    const raf_id_ref = useRef<number | undefined>(undefined);

    useEffect(() => {
        // Subscribe to event_bus — buffer events instead of triggering state per-event
        const unsubscribe = event_bus.subscribe_logs((entry) => {
            update_batch_ref.current.push(entry);
        });

        // Flush buffer to internal ring buffer on each animation frame
        const flush = () => {
            if (update_batch_ref.current.length > 0) {
                const batch = update_batch_ref.current;
                update_batch_ref.current = [];

                // Update ring buffer
                batch.forEach(entry => {
                    internal_buffer_ref.current[head_ref.current] = entry;
                    head_ref.current = (head_ref.current + 1) % MAX_LOG_ENTRIES;
                });

                // Trigger re-render
                set_render_version(v => v + 1);
            }
            raf_id_ref.current = requestAnimationFrame(flush);
        };
        raf_id_ref.current = requestAnimationFrame(flush);

        return () => {
            unsubscribe();
            if (raf_id_ref.current) cancelAnimationFrame(raf_id_ref.current);
        };
    }, []);

    /**
     * Reconstructs the log view from the circular buffer.
     * Memoized to prevent redundant array reconstruction on every render.
     */
    const active_logs = React.useMemo(() => {
        const result: log_entry[] = [];
        /* eslint-disable react-hooks/refs */
        for (let i = 0; i < MAX_LOG_ENTRIES; i++) {
            const idx = (head_ref.current + i) % MAX_LOG_ENTRIES;
            const entry = internal_buffer_ref.current[idx];
            if (entry) result.push(entry);
        }
        /* eslint-enable react-hooks/refs */
        return result;
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [render_version]);

    useEffect(() => {
        // Auto-scroll to bottom
        // PERFORMANCE: Use 'auto' instead of 'smooth' to prevent layout thrashing with large/rapid payloads.
        if (log_end_ref.current && typeof log_end_ref.current.scrollIntoView === 'function') {
            log_end_ref.current.scrollIntoView({ behavior: 'auto' });
        }
    }, [active_logs, is_open]);

    const [suggestions, set_suggestions] = useState<string[]>([]);
    const [suggestion_index, set_suggestion_index] = useState(-1);

    /** Memoize agent names for efficient autocomplete lookups. */
    const agent_names = React.useMemo(() => 
        (agents || []).map(a => a.name).filter(Boolean) as string[], 
        [agents]
    );

    /**
     * Processes keyboard inputs for the command line (e.g. Tab autocomplete).
     */
    const handle_key_down = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (file_suggestions.length > 0) {
            if (e.key === 'ArrowDown') {
                e.preventDefault();
                set_file_suggestion_index(prev => (prev + 1) % file_suggestions.length);
                return;
            }
            if (e.key === 'ArrowUp') {
                e.preventDefault();
                set_file_suggestion_index(prev => (prev - 1 + file_suggestions.length) % file_suggestions.length);
                return;
            }
            if (e.key === 'Enter' || e.key === 'Tab') {
                e.preventDefault();
                const selected = file_suggestions[file_suggestion_index];
                if (selected) {
                    complete_mention(selected);
                }
                return;
            }
            if (e.key === 'Escape') {
                e.preventDefault();
                set_file_suggestions([]);
                set_file_suggestion_index(-1);
                set_mention_start_index(-1);
                return;
            }
        }

        if (e.key === 'Tab') {
            e.preventDefault();

            const all_suggestions = [...AVAILABLE_COMMANDS, ...agent_names];

            if (suggestions.length === 0) {
                const current_input = input_value.toLowerCase();
                const matches = all_suggestions.filter(s => s && typeof s === 'string' && s.toLowerCase().startsWith(current_input));

                if (matches.length > 0) {
                    set_suggestions(matches);
                    set_suggestion_index(0);
                    set_input_value(matches[0]);
                }
            } else {
                const next_index = (suggestion_index + 1) % suggestions.length;
                set_suggestion_index(next_index);
                set_input_value(suggestions[next_index]);
            }
        } else if (e.key !== 'Shift') {
            // Reset suggestions on any other key press except shift
            set_suggestions([]);
            set_suggestion_index(-1);
        }
    };

    /**
     * Processes user input from the command line on Submit.
     */
    const handle_command_submit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!input_value.trim()) return;

        const command_text = input_value.trim();
        set_suggestions([]);
        set_suggestion_index(-1);

        // Emit USER command echo
        event_bus.emit_log({
            source: 'User',
            text: command_text,
            severity: 'info'
        });

        // Clear input immediately
        set_input_value('');

        // Delegate to the command processor with error boundary handling
        try {
            const result = await process_command(command_text, agents, is_safe_mode);

            if (result.should_clear_logs) {
                internal_buffer_ref.current.fill(null);
                head_ref.current = 0;
                set_render_version(v => v + 1);
            }
        } catch (error) {
            console.error('[Terminal] Command processing failed:', error);
            event_bus.emit_log({
                source: 'System',
                text: `Command execution failed: ${error instanceof Error ? error.message : String(error)}`,
                severity: 'error'
            });
        }
    };

    return (
        <div className={`fixed bottom-0 left-64 right-0 border-t border-[color:var(--color-border)] bg-[color:var(--color-background)] transition-all duration-300 flex flex-col z-50 ${is_open ? 'h-64' : 'h-10'}`}>
            <Tooltip content={i18n.t('terminal.tooltip')} position="top">
                <div
                    className="h-10 px-4 bg-[color:var(--color-surface)] flex items-center justify-between cursor-pointer hover:bg-zinc-800 transition-colors shrink-0 outline-none focus-visible:bg-zinc-800"
                    onClick={toggle_open}
                    onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && toggle_open()}
                    role="button"
                    tabIndex={0}
                    aria-expanded={is_open}
                    aria-label={i18n.t('terminal.toggle_aria')}
                >
                    <div className="flex items-center gap-2 text-xs font-mono text-zinc-400">
                        <Terminal_Icon size={14} />
                        <span>{i18n.t('terminal.label')}</span>
                    </div>
                    <div className="text-xs text-zinc-600 font-mono">
                        {is_open ? i18n.t('terminal.slide_down') : i18n.t('terminal.slide_up')}
                    </div>
                </div>
            </Tooltip>

            {/* Content (Only visible if open) */}
            {is_open && (
                <div className="flex-1 flex flex-col p-2 bg-black/50 font-mono text-xs overflow-hidden">
                    {/* Log Output */}
                    <div ref={log_container_ref} className="flex-1 overflow-y-auto space-y-1 mb-2 px-2 custom-scrollbar">
                        {active_logs.map((log) => (
                            <Log_Entry_Item key={log.id} log={log} />
                        ))}
                        <div ref={log_end_ref} />
                    </div>

                    {/* File Autocomplete Popup */}
                    {file_suggestions.length > 0 && (
                        <div className="absolute bottom-12 left-10 z-50 w-96 max-h-48 overflow-y-auto border border-zinc-800 bg-zinc-950/95 backdrop-blur-sm rounded shadow-xl flex flex-col custom-scrollbar">
                            {file_suggestions.map((file, idx) => (
                                <button
                                    type="button"
                                    key={file}
                                    className={`w-full text-left px-3 py-2 cursor-pointer text-zinc-300 font-mono text-[11px] flex items-center justify-between border-none transition-colors outline-none ${
                                        idx === file_suggestion_index 
                                            ? 'bg-green-500/10 text-green-400 border-l-2 border-green-500 pl-[10px]' 
                                            : 'hover:bg-zinc-900/60'
                                    }`}
                                    onClick={() => complete_mention(file)}
                                >
                                    <span className="truncate">{file}</span>
                                </button>
                            ))}
                        </div>
                    )}

                    {/* Input Line */}
                    <form onSubmit={handle_command_submit} className="flex items-center gap-2 px-2 border-t border-[color:var(--color-border)] pt-2 shrink-0">
                        <span className="text-green-500 font-bold">{i18n.t('terminal_prompt')}</span>
                        <input
                            ref={input_ref}
                            type="text"
                            data-terminal-input
                            className="flex-1 bg-transparent border-none outline-none text-zinc-200 placeholder-zinc-700 font-mono focus:ring-0"
                            placeholder={i18n.t('terminal.placeholder_command')}
                            aria-label={i18n.t('terminal.input_aria')}
                            value={input_value}
                            onChange={(e) => {
                                set_input_value(e.target.value);
                                check_mention(e.target.value, e.target.selectionStart ?? 0);
                            }}
                            onSelect={(e) => {
                                const target = e.target as HTMLInputElement;
                                check_mention(target.value, target.selectionStart ?? 0);
                            }}
                            onKeyDown={handle_key_down}
                            autoFocus
                        />
                    </form>
                </div>
            )}
        </div>
    );
}

// Metadata: [Terminal]
