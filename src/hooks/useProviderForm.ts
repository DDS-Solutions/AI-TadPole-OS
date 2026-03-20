import { type ProviderConfig } from '../stores/providerStore';

export interface PanelState {
    name: string;
    icon: string;
    apiKey: string;
    baseUrl: string;
    externalId: string;
    protocol: ProviderConfig['protocol'];
    customHeaders: string; // JSON string
    audioModel: string;
    persistToEngine: boolean;
    isTesting: boolean;
    testResult: 'idle' | 'success' | 'failed';
    testMessage: string;
}

export type Action =
    | { type: 'UPDATE_FIELD'; field: keyof PanelState; value: PanelState[keyof PanelState] };

export function panelReducer(state: PanelState, action: Action): PanelState {
    switch (action.type) {
        case 'UPDATE_FIELD':
            return { ...state, [action.field]: action.value };
        default:
            return state;
    }
}
