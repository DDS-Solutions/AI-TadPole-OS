/**
 * @module Terminal
 * Collapsible CLI-style interface fixed to the bottom of the dashboard.
 * Subscribes to the shared {@link EventBus} for real-time log display,
 * delegates slash-commands to the {@link processCommand} service,
 * and features intelligent tab-autocomplete for commands and agents.
 */
import { useState, useEffect, useRef } from 'react';
import { Terminal as TerminalIcon } from 'lucide-react';
import { EventBus } from '../services/eventBus';
import type { LogEntry } from '../services/eventBus';
import { useSettingsStore } from '../stores/settingsStore';
import { processCommand } from '../logic/commandProcessor';
import type { Agent } from '../types';
import { Tooltip } from './ui';
import { i18n } from '../i18n';

/** Maximum number of log entries to retain in the Terminal buffer. */
const MAX_LOG_ENTRIES = 500;

interface TerminalProps {
    /** Current agent list, passed to the command processor for lookups. */
    agents: Agent[];
}

/**
 * Collapsible terminal panel.
 * - Displays a scrollable log of all {@link LogEntry} events from the EventBus.
 * - Provides a `> _` command input for issuing slash-commands.
 * - Renders fixed at the bottom of the viewport, above the sidebar.
 *
 * Performance: Uses a RAF-flushed buffer to batch EventBus updates,
 * reducing re-renders from per-event to per-animation-frame.
 */
export default function TerminalComponent({ agents }: TerminalProps) {
    const [isOpen, setIsOpen] = useState(false); // Collapsible
    const [input, setInput] = useState('');
    const [logs, setLogs] = useState<LogEntry[]>([]);
    const logEndRef = useRef<HTMLDivElement>(null);
    const { settings } = useSettingsStore();
    const isSafeMode = settings.isSafeMode;

    /** Incoming event buffer, flushed to state on each animation frame. */
    const bufferRef = useRef<LogEntry[]>([]);

    useEffect(() => {
        // Subscribe to EventBus — buffer events instead of triggering state per-event
        const unsubscribe = EventBus.subscribe((entry) => {
            bufferRef.current.push(entry);
        });

        // Flush buffer to React state on each animation frame
        let rafId: number;
        const flush = () => {
            if (bufferRef.current.length > 0) {
                const batch = bufferRef.current;
                bufferRef.current = [];
                setLogs((prev) => {
                    const combined = [...prev, ...batch];
                    // Cap at MAX_LOG_ENTRIES to prevent unbounded memory growth
                    return combined.length > MAX_LOG_ENTRIES
                        ? combined.slice(-MAX_LOG_ENTRIES)
                        : combined;
                });
            }
            rafId = requestAnimationFrame(flush);
        };
        rafId = requestAnimationFrame(flush);

        return () => {
            unsubscribe();
            cancelAnimationFrame(rafId);
        };
    }, []);

    useEffect(() => {
        // Auto-scroll to bottom
        if (logEndRef.current) {
            logEndRef.current.scrollIntoView({ behavior: 'smooth' });
        }
    }, [logs, isOpen]);

    /**
     * Processes user input from the command line.
     * Emits the command to the EventBus and delegates to the command processor.
     */
    const [suggestions, setSuggestions] = useState<string[]>([]);
    const [suggestionIndex, setSuggestionIndex] = useState(-1);

    /**
     * Processes user input from the command line.
     */
    const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === 'Tab') {
            e.preventDefault();

            const availableCommands = ['/help', '/clear', '/pause', '/resume', '/kill', '/status', '/swarm status', '/swarm optimize'];
            const agentNames = agents.map(a => a.name).filter(Boolean) as string[];
            const allSuggestions = [...availableCommands, ...agentNames];

            if (suggestions.length === 0) {
                const currentInput = input.toLowerCase();
                const matches = allSuggestions.filter(s => s && typeof s === 'string' && s.toLowerCase().startsWith(currentInput));

                if (matches.length > 0) {
                    setSuggestions(matches);
                    setSuggestionIndex(0);
                    setInput(matches[0]);
                }
            } else {
                const nextIndex = (suggestionIndex + 1) % suggestions.length;
                setSuggestionIndex(nextIndex);
                setInput(suggestions[nextIndex]);
            }
        } else if (e.key !== 'Shift') {
            // Reset suggestions on any other key press except shift
            setSuggestions([]);
            setSuggestionIndex(-1);
        }
    };

    const handleCommand = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!input.trim()) return;

        const commandText = input.trim();
        setSuggestions([]);
        setSuggestionIndex(-1);

        // Emit USER command echo
        EventBus.emit({
            source: 'User',
            text: commandText,
            severity: 'info'
        });

        // Clear input immediately
        setInput('');

        // Delegate to the command processor
        const result = await processCommand(commandText, agents, isSafeMode);

        if (result.shouldClearLogs) {
            setLogs([]);
        }
    };

    return (
        <div className={`fixed bottom-0 left-64 right-0 border-t border-zinc-800 bg-zinc-950 transition-all duration-300 flex flex-col z-50 ${isOpen ? 'h-64' : 'h-10'}`}>
            {/* ... header ... */}
            <Tooltip content={i18n.t('terminal.tooltip')} position="top">
                <div
                    className="h-10 px-4 bg-zinc-900 flex items-center justify-between cursor-pointer hover:bg-zinc-800 transition-colors shrink-0"
                    onClick={() => setIsOpen(!isOpen)}
                >
                    <div className="flex items-center gap-2 text-xs font-mono text-zinc-400">
                        <TerminalIcon size={14} />
                        <span>{i18n.t('terminal.label')}</span>
                    </div>
                    <div className="text-xs text-zinc-600 font-mono">
                        {isOpen ? i18n.t('terminal.slide_down') : i18n.t('terminal.slide_up')}
                    </div>
                </div>
            </Tooltip>

            {/* Content (Only visible if open) */}
            {isOpen && (
                <div className="flex-1 flex flex-col p-2 bg-black/50 font-mono text-xs overflow-hidden">
                    {/* Log Output */}
                    <div className="flex-1 overflow-y-auto space-y-1 mb-2 px-2 custom-scrollbar">
                        {logs.map((log) => (
                            <div key={log.id} className="flex gap-2">
                                <span className="text-zinc-600">[{log.timestamp.toLocaleTimeString()}]</span>
                                <span className={`font-bold ${log.source === 'User' ? 'text-blue-400' :
                                    log.source === 'Agent' ? 'text-blue-400' :
                                        'text-zinc-400'
                                    }`}>
                                    {log.source === 'Agent' && log.agentId ? `${log.agentId}:` : `${log.source}:`}
                                </span>
                                <span className={`${log.severity === 'error' ? 'text-red-400' :
                                    log.severity === 'warning' ? 'text-yellow-400' :
                                        'text-zinc-300'
                                    }`}>{log.text}</span>
                            </div>
                        ))}
                        <div ref={logEndRef} />
                    </div>

                    {/* Input Line */}
                    <form onSubmit={handleCommand} className="flex items-center gap-2 px-2 border-t border-zinc-800 pt-2 shrink-0">
                        <span className="text-green-500 font-bold">{'>'}</span>
                        <input
                            type="text"
                            data-terminal-input
                            className="flex-1 bg-transparent border-none outline-none text-zinc-200 placeholder-zinc-700 font-mono focus:ring-0"
                            placeholder={i18n.t('terminal.placeholder_command')}
                            value={input}
                            onChange={(e) => setInput(e.target.value)}
                            onKeyDown={handleKeyDown}
                            autoFocus
                        />
                    </form>
                </div>
            )}
        </div>
    );
}
