/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Layout / tab_icons
 * - **Primary Entrypoints**: `TAB_ICONS`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import {
    LayoutDashboard,
    Users,
    MessagesSquare,
    Grid,
    Target,
    Cpu,
    Bot,
    Zap,
    Shield,
    Wrench,
    BarChart,
    Clock,
    Store,
    BookOpen,
    Settings,
    ShoppingBag,
    Lock,
    type LucideProps,
} from 'lucide-react';
import React from 'react';

export const TAB_ICONS: Record<string, React.ComponentType<LucideProps>> = {
    LayoutDashboard,
    Users,
    MessagesSquare,
    Grid,
    Target,
    Cpu,
    Bot,
    Zap,
    Shield,
    Wrench,
    BarChart,
    Clock,
    Store,
    BookOpen,
    Settings,
    ShoppingBag,
    Lock,
};
