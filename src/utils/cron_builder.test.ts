/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Utilities / cron_builder.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect } from 'vitest';
import { synthesize_interval_cron, synthesize_date_cron, describe_cron } from './cron_builder';

describe('cron_builder utility', () => {
    describe('synthesize_interval_cron', () => {
        it('synthesizes minute intervals correctly', () => {
            expect(synthesize_interval_cron({ unit: 'minutes', interval: 1 })).toBe('* * * * *');
            expect(synthesize_interval_cron({ unit: 'minutes', interval: 15 })).toBe('*/15 * * * *');
            expect(synthesize_interval_cron({ unit: 'minutes', interval: 30 })).toBe('*/30 * * * *');
        });

        it('synthesizes hour intervals with minute offsets', () => {
            expect(synthesize_interval_cron({ unit: 'hours', interval: 1, time: '00:00' })).toBe('0 * * * *');
            expect(synthesize_interval_cron({ unit: 'hours', interval: 2, time: '00:15' })).toBe('15 */2 * * *');
            expect(synthesize_interval_cron({ unit: 'hours', interval: 6, time: '00:45' })).toBe('45 */6 * * *');
        });

        it('synthesizes daily intervals with time', () => {
            expect(synthesize_interval_cron({ unit: 'days', interval: 1, time: '09:30' })).toBe('30 9 * * *');
            expect(synthesize_interval_cron({ unit: 'days', interval: 3, time: '14:00' })).toBe('0 14 */3 * *');
        });

        it('synthesizes monthly intervals with day and time', () => {
            expect(synthesize_interval_cron({ unit: 'months', interval: 1, day_of_month: 1, time: '00:00' })).toBe('0 0 1 * *');
            expect(synthesize_interval_cron({ unit: 'months', interval: 2, day_of_month: 15, time: '10:30' })).toBe('30 10 15 */2 *');
        });

        it('clamps negative or zero inputs gracefully', () => {
            expect(synthesize_interval_cron({ unit: 'minutes', interval: 0 })).toBe('* * * * *');
            expect(synthesize_interval_cron({ unit: 'minutes', interval: -5 })).toBe('* * * * *');
        });
    });

    describe('synthesize_date_cron', () => {
        it('synthesizes UTC calendar date correctly', () => {
            const cron = synthesize_date_cron({
                date: '2026-11-25',
                time: '14:30',
                timezone: 'utc'
            });
            expect(cron).toBe('30 14 25 11 *');
        });

        it('synthesizes local timezone calendar date correctly into UTC fields', () => {
            const cron = synthesize_date_cron({
                date: '2026-12-01',
                time: '10:00',
                timezone: 'local'
            });
            // Result should be valid 5 numbers/tokens
            const parts = cron.split(' ');
            expect(parts).toHaveLength(5);
            expect(parseInt(parts[0], 10)).toBeGreaterThanOrEqual(0);
            expect(parseInt(parts[1], 10)).toBeGreaterThanOrEqual(0);
        });

        it('handles fallback for missing date or time', () => {
            expect(synthesize_date_cron({ date: '', time: '', timezone: 'utc' })).toBe('0 0 1 1 *');
        });
    });

    describe('describe_cron', () => {
        it('describes standard shorthands', () => {
            expect(describe_cron('@hourly')).toContain('Runs every hour');
            expect(describe_cron('@daily')).toContain('Runs daily');
            expect(describe_cron('@weekly')).toContain('Runs weekly');
            expect(describe_cron('@monthly')).toContain('Runs on the 1st');
        });

        it('describes interval patterns', () => {
            expect(describe_cron('* * * * *')).toBe('Runs every minute');
            expect(describe_cron('*/15 * * * *')).toBe('Runs every 15 minute(s)');
            expect(describe_cron('0 * * * *')).toBe('Runs every hour at minute 0');
            expect(describe_cron('15 */2 * * *')).toBe('Runs every 2 hour(s) at minute 15');
            expect(describe_cron('30 9 * * *')).toBe('Runs daily at 09:30 (UTC)');
            expect(describe_cron('0 0 * * 0')).toBe('Runs weekly on Sunday at 00:00 (UTC)');
            expect(describe_cron('30 8 * * 1-5')).toBe('Runs Monday through Friday at 08:30 (UTC)');
            expect(describe_cron('0 14 */3 * *')).toBe('Runs every 3 day(s) at 14:00 (UTC)');
            expect(describe_cron('0 0 1 * *')).toBe('Runs monthly on day 1 at 00:00 (UTC)');
            expect(describe_cron('30 14 25 11 *')).toBe('Runs on Nov 25 at 14:30 (UTC)');
        });

        it('handles 6-part cron expressions with leading seconds', () => {
            expect(describe_cron('0 */15 * * * *')).toBe('Runs every 15 minute(s)');
            expect(describe_cron('0 30 9 * * *')).toBe('Runs daily at 09:30 (UTC)');
            expect(describe_cron('0 0 0 * * 0')).toBe('Runs weekly on Sunday at 00:00 (UTC)');
        });

        it('handles custom fallback', () => {
            expect(describe_cron('0 9-17 * * *')).toBe('Schedule: 0 9-17 * * *');
            expect(describe_cron('')).toBe('No schedule specified');
        });
    });
});
