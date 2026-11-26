/**
 * @docs ARCHITECTURE:UI
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Utilities / csv_parser.test
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect } from 'vitest';
import { parseCsvText } from './csv_parser';

describe('parseCsvText', () => {
    it('returns empty array for blank input', () => {
        expect(parseCsvText('')).toEqual([]);
        expect(parseCsvText('   \r\n  ')).toEqual([]);
    });

    it('parses RFC 4180 quoted fields, escaped quotes, and header auto-skip', () => {
        const csv = [
            'Title,Category,Price',
            '"Emergency ""24/7"" Drain, Clearing",Plumbing,149.00',
            '"Pipe \\"Pro\\" Sealant",Supplies,$19.50',
        ].join('\r\n');

        const items = parseCsvText(csv);
        expect(items).toHaveLength(2);
        expect(items[0]).toMatchObject({
            title: 'Emergency "24/7" Drain, Clearing',
            category: 'Plumbing',
            price: '$149.00',
        });
        expect(items[1]).toMatchObject({
            title: 'Pipe "Pro" Sealant',
            category: 'Supplies',
            price: '$19.50',
        });
    });

    it('supports two-column headerless rows with Imported category default', () => {
        const items = parseCsvText('Tune-Up,89.99');
        expect(items).toHaveLength(1);
        expect(items[0]).toMatchObject({
            title: 'Tune-Up',
            category: 'Imported',
            price: '$89.99',
        });
    });
});
