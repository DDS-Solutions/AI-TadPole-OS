/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **MCP Store Unit Tests**: Validates MCP card rendering states (installed vs. not installed),
 * and checks registry fetch resolution paths (successful fetch vs. fallback mock recovery).
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Unhandled fetch rejection in HTTP mocks, or missing connector properties.
 * - **Telemetry Link**: Search `[mcp_store.test]` in tracing logs.
 */

import '@testing-library/jest-dom';
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { MCP_Card } from './MCP_Card';
import { fetchMCPRegistry } from './mcp_store_api';

describe('MCP Store Component Group', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    afterEach(() => {
        vi.unstubAllGlobals();
    });

    describe('MCP_Card', () => {
        const mock_connector = {
            id: 'mcp-1',
            name: 'Slack Integration',
            description: 'Direct Slack Webhook poster tool',
            category: 'Communication',
            path: 'mcp/slack',
            version: '1.2.0',
            author: 'Tadpole Core',
            installed: false
        };

        it('renders uninstalled connector status', () => {
            render(<MCP_Card connector={mock_connector} />);

            expect(screen.getByText('Slack Integration')).toBeInTheDocument();
            expect(screen.getByText('Direct Slack Webhook poster tool')).toBeInTheDocument();
            expect(screen.getByText('v1.2.0')).toBeInTheDocument();
            expect(screen.getByText('Communication')).toBeInTheDocument();
            
            // Should show Install button
            expect(screen.getByRole('button', { name: /install/i })).toBeInTheDocument();
        });

        it('renders installed connector status', () => {
            const installed_connector = { ...mock_connector, installed: true };
            render(<MCP_Card connector={installed_connector} />);

            // Should show Installed text
            expect(screen.getByRole('button', { name: /installed/i })).toBeInTheDocument();
        });

        it('stops propagation when clicking the Install button', () => {
            const click_spy = vi.fn();
            render(
                <div onClick={click_spy}>
                    <MCP_Card connector={mock_connector} />
                </div>
            );

            const install_btn = screen.getByRole('button', { name: /install/i });
            fireEvent.click(install_btn);

            expect(click_spy).not.toHaveBeenCalled();
        });
    });

    describe('mcp_store_api', () => {
        it('fetchMCPRegistry returns data on successful response', async () => {
            const mock_registry = {
                connectors: [
                    { id: 'connector-a', name: 'Custom CRM' }
                ]
            };

            const mock_fetch = vi.fn().mockResolvedValue({
                ok: true,
                json: async () => mock_registry
            });
            vi.stubGlobal('fetch', mock_fetch);

            const result = await fetchMCPRegistry();

            expect(mock_fetch).toHaveBeenCalled();
            expect(result).toHaveLength(1);
            expect(result[0].name).toBe('Custom CRM');
        });

        it('fetchMCPRegistry returns fallback mockup list on request failure', async () => {
            const mock_fetch = vi.fn().mockRejectedValue(new Error('DNS Timeout'));
            vi.stubGlobal('fetch', mock_fetch);

            const result = await fetchMCPRegistry();

            expect(mock_fetch).toHaveBeenCalled();
            expect(result.length).toBeGreaterThan(0);
            expect(result[0].id).toBe('mcp-generic-crm');
        });
    });
});

// Metadata: [mcp_store_test]
