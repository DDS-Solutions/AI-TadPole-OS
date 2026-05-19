/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[tab_icons]` in observability traces.
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

// Metadata: [tab_icons]
