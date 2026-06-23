/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Assist Note
 * **Ticker Context**: Shared RAF-driven tick registry for the Neural Waterfall.
 * A single `setInterval` drives all subscribed row-level timers (O(1) listeners)
 * instead of N individual intervals per running span. (NW-008)
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Ticker not cleaning up → `clearInterval` never called if
 *   all subscribers unmount before the last `unsubscribe` fires.
 * - **Telemetry Link**: Search `[Neural_Waterfall]` in UI tracing.
 */

import React from 'react';

// CONSTANTS (IMR-02)
export const TICK_INTERVAL_MS = 500;
export const MIN_BAR_WIDTH_PX = 8;
export const ROW_HEIGHT_PX = 38;
export const ROW_TRACK_HEIGHT_PX = 32;
export const ROW_HEADER_WIDTH_PX = 128;
export const VIEWPORT_PADDING_PX = 160;

export type TickerContextValue = {
    subscribe: (listener: (now: number) => void) => () => void;
};

// Shared Ticker Registry Context (NW-008)
export const TickerContext = React.createContext<TickerContextValue | null>(null);

// Metadata: [WaterfallTicker]
