/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Pages / MCP_Store.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: `MCP_Store.test.tsx`
 */

import '@testing-library/jest-dom';
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import MCP_Store from './MCP_Store';
import * as mcpApi from '../components/mcp_store/mcp_store_api';

// Mock dependencies
vi.mock('../components/mcp_store/mcp_store_api', () => ({
    fetchMCPRegistry: vi.fn(),
}));

vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string, options?: { defaultValue?: string }) => options?.defaultValue || key,
    }
}));

describe('MCP_Store Page', () => {
    const mock_connectors = [
        {
            id: 'mcp-slack',
            name: 'Slack Integration',
            description: 'Post messages to Slack webhooks',
            category: 'Communication',
            path: 'mcp/slack',
            version: '1.0.0',
            author: 'Tadpole Core'
        },
        {
            id: 'mcp-github',
            name: 'GitHub Connector',
            description: 'Interact with GitHub issues and pull requests',
            category: 'DevTools',
            path: 'mcp/github',
            version: '2.1.0',
            author: 'Sovereign Engineering'
        }
    ];

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders loading state initially', () => {
        vi.mocked(mcpApi.fetchMCPRegistry).mockImplementation(() => new Promise(() => {}));

        render(<MCP_Store />);

        expect(screen.getByText(/loading/i)).toBeInTheDocument();
        expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('MCP Connector Store');
    });

    it('fetches and renders connector cards on success', async () => {
        vi.mocked(mcpApi.fetchMCPRegistry).mockResolvedValue(mock_connectors);

        render(<MCP_Store />);

        expect(await screen.findByText('Slack Integration')).toBeInTheDocument();
        expect(screen.getByText('GitHub Connector')).toBeInTheDocument();
        expect(screen.getByText(/2 Connectors Available/i)).toBeInTheDocument();
    });

    it('displays error message and handles retry when registry fetch fails', async () => {
        vi.mocked(mcpApi.fetchMCPRegistry)
            .mockRejectedValueOnce(new Error('Network Connection Refused'))
            .mockResolvedValueOnce(mock_connectors);

        render(<MCP_Store />);

        expect(await screen.findByText('Network Connection Refused')).toBeInTheDocument();
        
        const retryButton = screen.getByRole('button', { name: /retry/i });
        expect(retryButton).toBeInTheDocument();

        fireEvent.click(retryButton);

        expect(await screen.findByText('Slack Integration')).toBeInTheDocument();
    });

    it('displays empty state when no connectors are available', async () => {
        vi.mocked(mcpApi.fetchMCPRegistry).mockResolvedValue([]);

        render(<MCP_Store />);

        expect(await screen.findByText('No MCP connectors found in the registry.')).toBeInTheDocument();
    });
});
