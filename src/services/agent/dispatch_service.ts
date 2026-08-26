/**
 * @docs ARCHITECTURE:Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / dispatch_service
 * - **Primary Entrypoints**: `AgentTaskDispatchService`, `DispatchCommandInput`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import type { Task_Payload } from '../../contracts/agent';
import { api_request, map_api_error } from '../base_api_service';
import { PROVIDERS } from '../../constants';
import { use_provider_store } from '../../stores/provider_store';
import { use_vault_store } from '../../stores/vault_store';
import { use_model_store, type Model_Entry } from '../../stores/model_store';
import { event_bus } from '../event_bus';
import { track_operation } from '../../utils/telemetry';

export interface DispatchCommandInput {
    agent_id: string;
    message: string;
    model_id: string;
    provider: string;
    cluster_id?: string;
    department?: string;
    budget_usd?: number;
    external_id?: string;
    safe_mode?: boolean;
    analysis?: boolean;
    request_id?: string;
}

export class AgentTaskDispatchService {
    private readonly api_request_fn: typeof api_request;
    private readonly vault_store: typeof use_vault_store;
    private readonly model_store: typeof use_model_store;
    private readonly provider_store: typeof use_provider_store;
    private readonly event_bus_inst: typeof event_bus;

    constructor(
        api_request_fn: typeof api_request,
        vault_store: typeof use_vault_store,
        model_store: typeof use_model_store,
        provider_store: typeof use_provider_store,
        event_bus_inst: typeof event_bus
    ) {
        this.api_request_fn = api_request_fn;
        this.vault_store = vault_store;
        this.model_store = model_store;
        this.provider_store = provider_store;
        this.event_bus_inst = event_bus_inst;
    }

    public async checkPrerequisites(provider: string, agent_id: string): Promise<{ provider_api_key: string | null; is_actually_locked: boolean; warning?: string }> {
        const vault_store_state = this.vault_store.getState();
        const provider_api_key = await vault_store_state.get_api_key(provider);
        const is_actually_locked = !vault_store_state.is_unlocked();
        const is_local = provider === PROVIDERS.OLLAMA || provider === PROVIDERS.LOCAL;

        let warning: string | undefined;
        if (!provider_api_key && !is_local) {
            const reason = is_actually_locked ? 'Vault is Locked' : `No Key for ${provider.toUpperCase()}`;
            warning = `🔒 Neural Security: ${reason} for ${agent_id.toUpperCase()}.`;
        }

        return { provider_api_key, is_actually_locked, warning };
    }

    public buildCommandPayload(
        message: string,
        model_id: string,
        provider: string,
        provider_api_key: string | null,
        cluster_id?: string,
        department?: string,
        budget_usd?: number,
        external_id?: string,
        safe_mode?: boolean,
        analysis?: boolean
    ): Task_Payload {
        const body: Task_Payload = { message, cluster_id, department, provider, model_id, budget_usd, external_id, safe_mode, analysis, activeModelSlot: 'default' };

        if (provider_api_key) {
            body.api_key = provider_api_key;
            const model_store_state = this.model_store.getState();
            const inventory_model = model_store_state.models.find((m: Model_Entry) => m.name === model_id);
            if (inventory_model) {
                if (inventory_model.rpm) body.rpm = inventory_model.rpm;
                if (inventory_model.tpm) body.tpm = inventory_model.tpm;
                if (inventory_model.rpd) body.rpd = inventory_model.rpd;
                if (inventory_model.tpd) body.tpd = inventory_model.tpd;
            }
        }

        const base_url = this.provider_store.getState().base_urls[provider];
        if (base_url) {
            body.base_url = base_url;
        }

        return body;
    }

    public async send_command(input: DispatchCommandInput): Promise<boolean> {
        return track_operation('AgentAPI', `Dispatching command to agent: ${input.agent_id.toUpperCase()}`, async () => {
            try {
                const { provider_api_key, warning } = await this.checkPrerequisites(input.provider, input.agent_id);
                if (warning) {
                    this.event_bus_inst.emit_log({
                        source: 'System',
                        text: warning,
                        severity: 'warning'
                    });
                }
                const body = this.buildCommandPayload(
                    input.message,
                    input.model_id,
                    input.provider,
                    provider_api_key,
                    input.cluster_id,
                    input.department,
                    input.budget_usd,
                    input.external_id,
                    input.safe_mode,
                    input.analysis
                );

                await this.api_request_fn(`/v1/agents/${input.agent_id}/tasks`, {
                    method: 'POST',
                    body: JSON.stringify(body),
                    headers: input.request_id ? { 'X-Request-Id': input.request_id } : undefined
                });

                return true;
            } catch (err) {
                throw map_api_error(err);
            }
        }, { agent_id: input.agent_id, mission_id: input.cluster_id });
    }
}

// Metadata: [AgentTaskDispatchService]
// Telemetry: [AgentAPI]
