import { useEffect, useState } from 'react';
import { TadpoleOSSocket as socketService, type EngineHealthEvent } from '../services/socket';
import { useThrottledStatus } from './useThrottledStatus';

export interface EngineStats {
    uptime: string;
    cpu: number;
    memory: number;
    latency: number;
}

type ConnectionStatus = 'connected' | 'disconnected' | 'reconnecting';

export const useEngineStatus = () => {
    const [status, setStatus] = useState<ConnectionStatus>('disconnected');
    const [stats, setStats] = useThrottledStatus<EngineStats>({
        uptime: '0s',
        cpu: 0,
        memory: 0,
        latency: 0,
    }, 150); // Throttle to 150ms for smooth UI updates

    // Swarm telemetry fields driven by health payloads
    const [activeAgents, setActiveAgents] = useState(0);
    const [maxDepth, setMaxDepth] = useState(3);
    const [tpm, setTpm] = useState(0);
    const [recruitCount, setRecruitCount] = useState(0);

    useEffect(() => {
        const unsubscribe = socketService.subscribeHealth((data: EngineHealthEvent) => {
            const uptimeNum = data.uptime ?? 0;
            const uptimeStr = uptimeNum >= 3600
                ? `${Math.floor(uptimeNum / 3600)}h`
                : uptimeNum >= 60
                    ? `${Math.floor(uptimeNum / 60)}m`
                    : `${uptimeNum}s`;
            setStats({
                uptime: uptimeStr,
                cpu: (data as Record<string, unknown>).cpu as number ?? 0,
                memory: (data as Record<string, unknown>).memory as number ?? 0,
                latency: (data as Record<string, unknown>).latency as number ?? 0,
            });
            const ext = data as Record<string, unknown>;
            if (ext.activeAgents !== undefined) setActiveAgents(ext.activeAgents as number);
            if (ext.maxDepth !== undefined) setMaxDepth(ext.maxDepth as number);
            if (ext.tpm !== undefined) setTpm(ext.tpm as number);
            if (ext.recruitCount !== undefined) setRecruitCount(ext.recruitCount as number);
        });

        const statusUnsubscribe = socketService.subscribeStatus((newStatus: string) => {
            setStatus(newStatus as ConnectionStatus);
        });

        return () => {
            unsubscribe();
            statusUnsubscribe();
        };
    }, [setStats]);

    // Derived convenience properties used across consumers
    const isOnline = status === 'connected';
    const connectionState = isOnline ? 'Connected' : status === 'reconnecting' ? 'Reconnecting' : 'Disconnected';

    return {
        status,
        stats,
        // Flat convenience properties
        isOnline,
        cpu: stats.cpu,
        memory: stats.memory,
        latency: stats.latency,
        connectionState,
        activeAgents,
        maxDepth,
        tpm,
        recruitCount,
    };
};
