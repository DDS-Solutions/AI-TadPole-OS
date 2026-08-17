/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Assist Note
 * Regression coverage for the adjacent production module and its public contracts.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Contract, rendering, state transition, or error-handling regression.
 * - **Trace Scope**: Vitest assertions and test-local mocks.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Hardware_Load } from './Hardware_Load';
import * as apiModule from '../../services/base_api_service';

describe('Hardware_Load Component', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
    });

    it('renders compute profile metrics when API returns load data', async () => {
        vi.spyOn(apiModule, 'api_request').mockResolvedValue({
            cpu_usage: 24.5,
            memory_used: 4 * 1024 * 1024 * 1024,
            memory_total: 16 * 1024 * 1024 * 1024,
            active_processes: 12,
            gpu_usage: null
        });

        render(<Hardware_Load />);

        expect(await screen.findByText('24.5%')).toBeDefined();
        expect(await screen.findByText('4.0')).toBeDefined();
        expect(await screen.findByText('12')).toBeDefined();
    });

    it('renders error banner when API fails', async () => {
        vi.spyOn(apiModule, 'api_request').mockRejectedValue(new Error('Hardware stats connection failed'));

        render(<Hardware_Load />);

        expect(await screen.findByText('Hardware stats connection failed')).toBeDefined();
    });
});
