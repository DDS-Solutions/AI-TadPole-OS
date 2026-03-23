import React from 'react';
import { Users, Network, TrendingUp, Layers } from 'lucide-react';
import { i18n } from '../i18n';

interface TelemetryCardProps {
    label: string;
    value: string | number;
    subtext: string;
    icon: React.ElementType;
    status: 'low' | 'normal' | 'high';
    colorClass: string;
}

const TelemetryCard = ({ label, value, subtext, icon: Icon, status, colorClass }: TelemetryCardProps) => {
    const statusGlow = {
        low: 'shadow-[0_0_15px_rgba(16,185,129,0.1)] border-emerald-500/10',
        normal: 'shadow-[0_0_15px_rgba(234,179,8,0.1)] border-yellow-500/10',
        high: 'shadow-[0_0_15px_rgba(239,68,68,0.2)] border-red-500/30'
    }[status];

    const statusText = {
        low: 'text-emerald-500/70',
        normal: 'text-yellow-500/70',
        high: 'text-red-500/90 font-bold animate-pulse'
    }[status];

    return (
        <div className={`p-4 rounded-2xl border bg-zinc-900/40 backdrop-blur-md transition-all duration-500 hover:bg-zinc-900/60 ${statusGlow} group relative overflow-hidden`}>
            {/* Background Accent Gradient */}
            <div className={`absolute -right-4 -top-4 w-24 h-24 bg-current opacity-[0.03] blur-2xl group-hover:opacity-[0.06] transition-opacity ${colorClass}`} />

            <div className="flex justify-between items-start mb-2 relative z-10">
                <span className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em]">{label}</span>
                <Icon size={14} className={`${colorClass} opacity-40 group-hover:opacity-100 transition-opacity`} />
            </div>

            <div className={`text-2xl font-mono tracking-tighter mb-1 relative z-10 ${status === 'high' ? colorClass : 'text-zinc-100'}`}>
                {value}
            </div>

            <div className="flex items-center gap-2 relative z-10">
                <span className={`text-[9px] uppercase font-bold tracking-widest font-mono ${statusText}`}>{status}</span>
                <span className="text-[9px] text-zinc-600 font-medium truncate">{subtext}</span>
            </div>
        </div>
    );
};

interface SwarmTelemetryProps {
    activeAgents: number;
    maxDepth: number;
    tpm: number;
    recruitCount: number;
    maxDensity: number;
}

export const SwarmTelemetry = ({ activeAgents, maxDepth, tpm, recruitCount, maxDensity }: SwarmTelemetryProps) => {
    const densityVal = Math.round((activeAgents / maxDensity) * 100);

    // Status Logic
    const densityStatus = densityVal > 80 ? 'high' : densityVal > 30 ? 'normal' : 'low';
    const depthStatus = maxDepth > 4 ? 'high' : maxDepth > 2 ? 'normal' : 'low';
    const velocityStatus = recruitCount > 3 ? 'high' : recruitCount > 0 ? 'normal' : 'low';

    // Mock Fiscal logic (assuming $0.002 average per 1k tokens)
    const estimatedCost = (tpm / 1000) * 0.002 * 60; // Est $/hr
    const fiscalStatus = estimatedCost > 5 ? 'high' : estimatedCost > 1 ? 'normal' : 'low';

    return (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 w-full">
            <TelemetryCard
                label={i18n.t('telemetry.swarm_density')}
                value={`${densityVal}%`}
                subtext={i18n.t('telemetry.density_subtext', { active: activeAgents, max: maxDensity })}
                icon={Users}
                status={densityStatus}
                colorClass="text-emerald-400"
            />
            <TelemetryCard
                label={i18n.t('telemetry.logic_depth')}
                value={maxDepth}
                subtext={i18n.t('telemetry.depth_subtext')}
                icon={Layers}
                status={depthStatus}
                colorClass="text-cyan-400"
            />
            <TelemetryCard
                label={i18n.t('telemetry.swarm_velocity')}
                value={recruitCount}
                subtext={i18n.t('telemetry.velocity_subtext')}
                icon={Network}
                status={velocityStatus}
                colorClass="text-amber-400"
            />
            <TelemetryCard
                label={i18n.t('telemetry.fiscal_burn')}
                value={`$${estimatedCost.toFixed(2)}`}
                subtext={i18n.t('telemetry.fiscal_subtext')}
                icon={TrendingUp}
                status={fiscalStatus}
                colorClass="text-rose-400"
            />
        </div>
    );
};
