/**
 * @docs ARCHITECTURE:UI
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Utilities / command_table_utils.test
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect } from 'vitest';
import { calculate_utilization, format_cost_usd } from './command_table_utils';

describe('command_table_utils', () => {
    it('calculates utilization safely including zero-budget edge cases', () => {
        expect(calculate_utilization(0.5, 2)).toBe(25);
        expect(calculate_utilization(0, 0)).toBe(0);
        expect(calculate_utilization(0.01, 0)).toBe(100);
    });

    it('formats sub-cent micro-costs in scientific notation and standard costs to 4 decimals', () => {
        expect(format_cost_usd(0.00005)).toBe('5.00e-5');
        expect(format_cost_usd(1.23456)).toBe('1.2346');
        expect(format_cost_usd(0)).toBe('0.0000');
    });
});
