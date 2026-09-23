/**
 * @docs ARCHITECTURE:Infrastructure:Execution
 *
 * ### AI Context Alignment
 * - **Subsystem**: Release Governance / Unified Test Entrypoint
 * - **Primary Entrypoints**: Node process
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - [Structural] All release-governance witnesses execute in one process for sandbox portability.
 *
 * ### 🔍 Debugging & Observability
 * - **Witness Targets**: version authority, changelog guard, provenance, and toolchain alignment
 */

'use strict';

require('./bump_version.node-test.cjs');
require('./check_changelog.node-test.cjs');
require('./generate_release_provenance.node-test.cjs');
require('./doctor.node-test.cjs');
