import { Component, type ErrorInfo, type ReactNode } from 'react';
import { AlertCircle, RefreshCcw } from 'lucide-react';

interface Props {
    children: ReactNode;
    fallback?: ReactNode;
    name?: string;
}

interface State {
    hasError: boolean;
    error: Error | null;
}

/**
 * Standard React Error Boundary to catch UI crashes.
 * Displays a neural-themed recovery interface.
 */
export default class Error_Boundary extends Component<Props, State> {
    public state: State = {
        hasError: false,
        error: null
    };

    public static getDerivedStateFromError(error: Error): State {
        return { hasError: true, error };
    }

    public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
        console.error(`[Neural Sector Fault] ${this.props.name || 'Unknown'}:`, error, errorInfo);
    }

    private handleReset = () => {
        this.setState({ hasError: false, error: null });
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
                        Neural Sector Fault: {this.props.name || 'Core'}
                    </h2>
                    <p className="text-red-400/60 text-xs font-mono mb-6 text-center max-w-md">
                        {this.state.error?.message || 'A critical rendering exception was detected in this sector.'}
                    </p>
                    <button
                        onClick={this.handleReset}
                        className="flex items-center gap-2 px-6 py-2 bg-red-900/40 hover:bg-red-900/60 text-red-200 text-xs font-bold uppercase tracking-widest rounded-full border border-red-500/30 transition-all active:scale-95"
                    >
                        <RefreshCcw size={14} />
                        Re-initialize Sector
                    </button>
                    <div className="mt-8 pt-4 border-t border-red-900/20 w-full text-center">
                        <span className="text-[10px] font-mono text-red-900 uppercase tracking-widest">TadpoleOS v3.2 Safety Protocol Active</span>
                    </div>
                </div>
            );
        }

        return this.props.children;
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

