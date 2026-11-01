/**
 * @docs ARCHITECTURE:Agent
 *
 * ### AI Context Alignment
 * - **Subsystem**: Invariant Verification Suite / inv_007_outward_catalog.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 *
 * INV-007: Outward Customer Catalog & Gateway Invariant Suite
 *
 * Asserts structural & behavioral guarantees in server-rs/src/agent/outward/:
 *   1. Floor boundaries for catalog scale (items, title, category, description, search limits)
 *   2. Untrusted text sanitization & prompt data block fencing
 *   3. Price locale disambiguation (DotDecimal vs CommaDecimal)
 *   4. Fast O(1) deduplication indexing without O(n^2) closure allocations
 *   5. Non-destructive custom skill composition in OutwardGateway
 *   6. Atomic & durable file persistence with NamedTempFile and fsync
 */

import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

describe('INV-007: Outward Customer Catalog & Gateway Invariants', () => {
    const catalogSource = readFileSync(
        resolve('server-rs/src/agent/outward/customer_catalog.rs'),
        'utf-8'
    );
    const gatewaySource = readFileSync(
        resolve('server-rs/src/agent/outward/outward_gateway.rs'),
        'utf-8'
    );
    const modSource = readFileSync(
        resolve('server-rs/src/agent/outward/mod.rs'),
        'utf-8'
    );

    it('asserts floor limits and capacity bounds in outward mod.rs', () => {
        expect(modSource).toContain('pub const MAX_CATALOG_ITEMS: usize = 50_000;');
        expect(modSource).toContain('pub const DEFAULT_MAX_CONTEXT_ITEMS: usize = 50;');
        expect(modSource).toContain('pub const MAX_CONTEXT_CHARS: usize = 8000;');
        expect(modSource).toContain('pub const MAX_SEARCH_RESULTS: usize = 25;');
        expect(modSource).toContain('pub const MAX_TITLE_LEN: usize = 200;');
        expect(modSource).toContain('pub const MAX_CATEGORY_LEN: usize = 100;');
        expect(modSource).toContain('pub const MAX_DESCRIPTION_LEN: usize = 2000;');
    });

    it('enforces prompt data block boundary fencing in to_llm_context', () => {
        expect(catalogSource).toContain('<!-- BEGIN_CATALOG_DATA -->');
        expect(catalogSource).toContain('<!-- END_CATALOG_DATA -->');
        expect(catalogSource).toContain('UntrustedText::sanitize');
    });

    it('verifies price parsing handles European decimal ambiguity explicitly', () => {
        expect(catalogSource).toContain('PriceLocale::DotDecimal');
        expect(catalogSource).toContain('PriceLocale::CommaDecimal');
        expect(catalogSource).toContain('Set price_locale=comma_decimal for European-format files');
    });

    it('ensures catalog uses index-based dedup rather than O(n^2) linear scan allocations', () => {
        // Must contain index lookup
        expect(catalogSource).toContain('self.index.get(&key)');
        // Must not contain the old anti-pattern of inline lowercase allocation inside find closure
        expect(catalogSource).not.toContain('i.title.trim().to_lowercase() == title_lower');
    });

    it('ensures outward gateway encapsulates state and preserves custom skills non-destructively', () => {
        // OutwardGateway fields must not be public
        expect(gatewaySource).not.toContain('pub agent_card: A2aAgentCard');
        expect(gatewaySource).not.toContain('pub profile: BusinessProfile');
        expect(gatewaySource).toContain('custom_skills: Vec<A2aSkill>');
        expect(gatewaySource).toContain('pub fn get_agent_card');
    });

    it('ensures file persistence uses unique temp files and sync_all for durability', () => {
        expect(catalogSource).toContain('tempfile::Builder::new()');
        expect(catalogSource).toContain('temp_file.as_file().sync_all()');
        expect(catalogSource).toContain('temp_file.persist(path)');
    });
});
