/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Intelligence / WaterfallTicker
 * - **Primary Entrypoints**: `TICK_INTERVAL_MS`, `MIN_BAR_WIDTH_PX`, `ROW_HEIGHT_PX`, `ROW_TRACK_HEIGHT_PX`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
// Telemetry: [Neural_Waterfall]
