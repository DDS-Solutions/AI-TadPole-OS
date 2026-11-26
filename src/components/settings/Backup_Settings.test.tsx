/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Settings / Backup_Settings.test
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { Backup_Settings } from './Backup_Settings';
import type { Tadpole_Settings } from '../../stores/settings_store';

const baseSettings: Tadpole_Settings = {
    tadpole_os_url: 'http://127.0.0.1:8000',
    tadpole_os_api_key: 'test-admin-token',
    backup_mode: 'local',
    backup_path: 'backups/local',
    backup_enabled: true,
} as Tadpole_Settings;

describe('Backup_Settings', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
    });

    afterEach(() => {
        vi.unstubAllGlobals();
    });

    it('renders FAILED status badge when server reports FAILED and does not claim HEALTHY', async () => {
        vi.stubGlobal(
            'fetch',
            vi.fn().mockResolvedValue({
                ok: true,
                json: async () => ({
                    enabled: true,
                    destination_type: 'local',
                    destination_path: 'backups/local',
                    has_passphrase: true,
                    last_run_timestamp: 1700000000,
                    last_status: 'FAILED: disk full',
                    total_snapshots: 1,
                    total_size_bytes: 1024 * 1024,
                    retention_hours: 48,
                }),
            })
        );

        render(
            <Backup_Settings
                settings={baseSettings}
                handle_change={vi.fn()}
                update_setting={vi.fn()}
            />
        );

        await waitFor(() => {
            expect(screen.getByText('FAILED')).toBeInTheDocument();
        });
        expect(screen.queryByText('HEALTHY')).not.toBeInTheDocument();
    });

    it('surfaces RFC 9457 detail error from /configure and aborts /trigger', async () => {
        const fetchMock = vi.fn()
            // Initial status fetch
            .mockResolvedValueOnce({
                ok: true,
                json: async () => ({
                    enabled: false,
                    destination_type: 'local',
                    destination_path: '',
                    has_passphrase: false,
                    last_run_timestamp: null,
                    last_status: null,
                    total_snapshots: 0,
                    total_size_bytes: 0,
                    retention_hours: 48,
                }),
            })
            // /configure fails with 403 RFC 9457 detail
            .mockResolvedValueOnce({
                ok: false,
                status: 403,
                json: async () => ({
                    title: 'Forbidden',
                    detail: 'Backup path must reside within an authorized backup directory.',
                }),
            });

        vi.stubGlobal('fetch', fetchMock);

        render(
            <Backup_Settings
                settings={baseSettings}
                handle_change={vi.fn()}
                update_setting={vi.fn()}
            />
        );

        const triggerBtn = await screen.findByRole('button', { name: /Backup Now/i });
        fireEvent.click(triggerBtn);

        await waitFor(() => {
            expect(
                screen.getByText('Backup path must reside within an authorized backup directory.')
            ).toBeInTheDocument();
        });

        // Must NOT call /trigger when /configure rejected the request
        expect(fetchMock).toHaveBeenCalledTimes(2);
    });

    it('surfaces 403 admin requirement on initial status fetch and executes happy-path backup trigger', async () => {
        const fetchMock = vi.fn()
            // Initial status returns 403
            .mockResolvedValueOnce({
                ok: false,
                status: 403,
                json: async () => ({
                    detail: 'Administrative privileges required',
                }),
            })
            // /configure succeeds
            .mockResolvedValueOnce({
                ok: true,
                json: async () => ({ enabled: true, has_passphrase: true }),
            })
            // /trigger succeeds
            .mockResolvedValueOnce({
                ok: true,
                json: async () => ({
                    success: true,
                    snapshot_filename: 'tadpole-snapshot-20261120.tadpole-snap.enc',
                }),
            })
            // Post-trigger status refresh returns HEALTHY
            .mockResolvedValueOnce({
                ok: true,
                json: async () => ({
                    enabled: true,
                    destination_type: 'local',
                    destination_path: 'backups/local',
                    has_passphrase: true,
                    last_run_timestamp: 1700000000,
                    last_status: 'HEALTHY',
                    total_snapshots: 2,
                    total_size_bytes: 2 * 1024 * 1024,
                    retention_hours: 48,
                }),
            });

        vi.stubGlobal('fetch', fetchMock);

        render(
            <Backup_Settings
                settings={baseSettings}
                handle_change={vi.fn()}
                update_setting={vi.fn()}
            />
        );

        await waitFor(() => {
            expect(screen.getByText('Administrative privileges required')).toBeInTheDocument();
        });

        const triggerBtn = screen.getByRole('button', { name: /Backup Now/i });
        fireEvent.click(triggerBtn);

        await waitFor(() => {
            expect(screen.getByText(/tadpole-snapshot-20261120\.tadpole-snap\.enc/)).toBeInTheDocument();
            expect(screen.getByText('HEALTHY')).toBeInTheDocument();
        });
        expect(fetchMock).toHaveBeenCalledTimes(4);
    });
});
