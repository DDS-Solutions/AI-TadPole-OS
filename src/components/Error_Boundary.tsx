/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Fail-safe wrapper for React component tree isolation. 
 * Intercepts "Neural Sector Faults" (rendering crashes), logs them to the system bus, and provides a recovery portal for sector re-initialization.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Failure to catch async errors (standard React limitation), infinite retry loops if the underlying fault persists after reset, or CSS animation blocking.
 * - **Telemetry Link**: Search for `[Neural Sector Fault]` or `Error_Boundary` in browser logs.
 */

import { Component, type ErrorInfo, type ReactNode } from 'react';
import { AlertCircle, RefreshCcw } from 'lucide-react';
import { i18n } from '../i18n';
import { event_bus } from '../services/event_bus';

interface Props {
    children: ReactNode;
    fallback?: ReactNode;
    name?: string;
}

interface State {
    hasError: boolean;
    error: Error | null;
    reset_key: number;
}

/**
 * Standard React Error Boundary to catch UI crashes.
 * Displays a neural-themed recovery interface.
 */
export default class Error_Boundary extends Component<Props, State> {
    public state: State = {
        hasError: false,
        error: null,
        reset_key: 0
    };

    public static getDerivedStateFromError(error: Error): Partial<State> {
        return { hasError: true, error };
    }

    /**
     * Catches and logs rendering errors within a specific UI sector.
     * 
     * ### 🛡️ Stability: Neural Sector Fault Isolation
     * If a component in the tree crashes (e.g., due to malformed LLM response data), 
     * this boundary prevents the entire dashboard from going blank. It logs 
     * the fault to the console with the `[Neural Sector Fault]` tag for 
     * rapid telemetry tracing.
     */
    public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
        console.error(`[Neural Sector Fault] ${this.props.name || 'Unknown'}:`, error, errorInfo);
        
        // Telemetry Injection: Broadcast fault to the system bus
        event_bus.emit_log({
            source: 'System',
            text: `[Neural Sector Fault] ${this.props.name || 'Core'}: ${error.message}`,
            severity: 'error'
        });
    }

    private handleReset = () => {
        this.setState(s => ({ 
            hasError: false, 
            error: null,
            reset_key: s.reset_key + 1
        }));
    };

    public render() {
        if (this.state.hasError) {
            if (this.props.fallback) return this.props.fallback;

            return (
                <div className="flex flex-col items-center justify-center p-8 border border-red-900/50 bg-red-950/10 rounded-3xl backdrop-blur-sm animate-in fade-in zoom-in duration-300">
                    <div className="p-4 bg-red-900/20 rounded-full mb-4">
                        <AlertCircle className="w-8 h-8 text-red-500" />
                    </div>
                    <h2 className="text-xl font-bold text-red-100 mb-2 truncate max-w-full">
                        {i18n.t('system.neural_fault', { name: this.props.name || 'Core' })}
                    </h2>
                    <p className="text-red-400/60 text-xs font-mono mb-6 text-center max-w-md">
                        {this.state.error?.message || i18n.t('system.critical_exception')}
                    </p>
                    <button
                        onClick={this.handleReset}
                        className="flex items-center gap-2 px-6 py-2 bg-red-900/40 hover:bg-red-900/60 text-red-200 text-xs font-bold uppercase tracking-widest rounded-full border border-red-500/30 transition-all active:scale-95"
                    >
                        <RefreshCcw size={14} />
                        {i18n.t('system.reinitialize_sector')}
                    </button>
                    <div className="mt-8 pt-4 border-t border-red-900/20 w-full text-center">
                        <span className="text-[10px] font-mono text-red-900 uppercase tracking-widest">
                            {i18n.t('system.safety_protocol', { version: '3.2' })}
                        </span>
                    </div>
                </div>
            );
        }

        return (
            <div key={this.state.reset_key} className="contents">
                {this.props.children}
            </div>
        );
    }
}

/**
 * Specialized boundary for smaller dashboard widgets.
 */
export const SectorBoundary = ({ children, name }: { children: ReactNode, name: string }) => (
    <Error_Boundary name={name}>
        {children}
    </Error_Boundary>
);

// Metadata: [Error_Boundary]

// Metadata: [Error_Boundary]
