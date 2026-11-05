/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Types / mission_dossier
 * - **Primary Entrypoints**: `MissionDossier`, `MissionSummaryItem`, `MerkleStepRecord`, `CasDiffRecord`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Fully aligned with server-rs routes::missions data contracts.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

export interface MissionSummaryItem {
    id: string;
    agent_id: string;
    title: string;
    status: string;
    budget_usd: number;
    cost_usd: number;
    is_pinned: boolean;
    created_at: string;
    updated_at: string;
    log_count: number;
    diff_count: number;
}

export interface CatalogResponse {
    success: boolean;
    data: MissionSummaryItem[];
    total: number;
    page: number;
    per_page: number;
}

export interface MerkleStepRecord {
    id: string;
    mission_id: string;
    step_index?: number;
    agent_id: string;
    source: string;
    text: string;
    severity: 'info' | 'success' | 'warning' | 'error' | string;
    timestamp: string;
    metadata: Record<string, unknown> | null;
    hash: string | null;
    prev_hash: string | null;
    is_verified: boolean;
}

export interface CasDiffSummary {
    additions: number;
    deletions: number;
    changes: number;
}

export interface CasDiffRecord {
    id: number;
    file_path: string;
    version_num: number;
    hash: string;
    size_bytes: number;
    created_at: string;
    pre_mutation_content: string;
    current_content: string;
    diff_summary: CasDiffSummary;
}

export interface SwarmFindingRecord {
    id: string;
    mission_id: string;
    agent_id: string;
    topic: string;
    finding: string;
    timestamp: string;
}

export interface TrustGraphNode {
    id: string;
    name: string;
    node_type: string;
    description: string | null;
    mission_id: string | null;
    created_at: number;
}

export interface TrustGraphEdge {
    id: string;
    source_entity_id: string;
    target_entity_id: string;
    relation_type: string;
    weight: number;
    mission_id: string | null;
    created_at: number;
}

export interface TrustGraphPayload {
    entities: TrustGraphNode[];
    relations: TrustGraphEdge[];
}

export interface TransactionLedgerRecord {
    id: string;
    transaction_type: string;
    buyer_address: string | null;
    seller_address: string | null;
    asset_id: string | null;
    amount: number;
    status: string;
    audit_signature: string | null;
    created_at: number;
}

export interface EconomicsPayload {
    total_cost_usd: number;
    budget_usd: number;
    total_tokens?: number;
    transactions: TransactionLedgerRecord[];
}

export interface DebriefPayload {
    agent99_notes: string | null;
    drift_score: number | null;
    verdict: string | null;
    recommendations: string[];
}

export interface MissionDossier {
    mission: MissionSummaryItem;
    logs: MerkleStepRecord[];
    diffs: CasDiffRecord[];
    memory_palace: SwarmFindingRecord[];
    trust_graph: TrustGraphPayload;
    economics: EconomicsPayload;
    debrief: DebriefPayload;
}

export type DossierTabKey = 
    | 'merkle' 
    | 'cas' 
    | 'memory' 
    | 'trustgraph' 
    | 'economics' 
    | 'debrief';
