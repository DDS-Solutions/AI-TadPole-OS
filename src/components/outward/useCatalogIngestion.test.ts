/**
 * @docs ARCHITECTURE:UI
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Outward / useCatalogIngestion.test
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { useCatalogIngestion } from './useCatalogIngestion';
import * as baseApi from '../../services/base_api_service';

describe('useCatalogIngestion', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
    });

    it('surfaces server sync failure instead of faking success when CSV import API rejects', async () => {
        vi.spyOn(baseApi, 'api_request').mockRejectedValue(new Error('Forbidden: Administrative privileges required'));
        const onItemsIngested = vi.fn();
        const { result } = renderHook(() => useCatalogIngestion({ onItemsIngested, companyKey: 'Mock-1-Product' }));

        const csvContent = 'title,category,price\nWidget Pro,Tools,$19.99\n';
        const file = new File([csvContent], 'catalog.csv', { type: 'text/csv' });

        await act(async () => {
            result.current.handleCsvFileChange({
                target: { files: [file] },
            } as unknown as React.ChangeEvent<HTMLInputElement>);
        });

        await waitFor(() => {
            expect(result.current.importStatus).toContain('server sync failed');
            expect(result.current.importStatus).toContain('Administrative privileges required');
        });
        expect(onItemsIngested).toHaveBeenCalledTimes(1);
    });

    it('reports PDF ingestion unavailable cleanly', () => {
        const { result } = renderHook(() => useCatalogIngestion({ onItemsIngested: vi.fn() }));
        const file = new File(['%PDF-1.4'], 'pricelist.pdf', { type: 'application/pdf' });
        const target = { files: [file], value: 'pricelist.pdf' };

        act(() => {
            result.current.handlePdfFileChange({ target } as unknown as React.ChangeEvent<HTMLInputElement>);
        });

        expect(result.current.importStatus).toContain('PDF ingestion is unavailable');
        expect(target.value).toBe('');
    });
});
