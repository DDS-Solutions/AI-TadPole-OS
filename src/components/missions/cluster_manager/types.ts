/**
 * @docs ARCHITECTURE:Interface:Missions
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / Cluster Manager / Types
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import type { Agent } from '../../../types';
import type { Mission_Cluster } from '../../../stores/workspace_store';

export interface ClusterManagerModalProps {
    isOpen: boolean;
    onClose: () => void;
    agents: Agent[];
}

export interface PresetFormData {
    editing_id: string | null;
    name: string;
    description: string;
    department: Mission_Cluster['department'];
    theme: Mission_Cluster['theme'];
    budget_usd: string;
    badge_label: string;
    selected_agents: string[];
}

export const INITIAL_FORM_STATE: PresetFormData = {
    editing_id: null,
    name: '',
    description: '',
    department: 'Engineering',
    theme: 'blue',
    budget_usd: '1500',
    badge_label: '',
    selected_agents: []
};
