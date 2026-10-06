> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Core Directives / deploy_to_prod
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[deploy_to_prod]`)

# 🚀 Directive: Deploy to Prod (SOP-SYS-02)

## 🎯 Primary Objective
Govern the final transition of code from staging to the "Sovereign" production environment. Zero-downtime and 100% reliability are the target outcomes.

---

## 🛫 Pre-Deployment Checklist
- [ ] **STASIS Check** *(IDENTITY.md Directive #7)*: Verify system is **NOT** in `STASIS` mode. If `STASIS` is active, deployment is **blocked** until Entity 0 issues an explicit `RESUME` command.
- [ ] **Artifact Verification**: Build must pass on `ci_cd_pipeline.md`.
- [ ] **Data Safety**: Persistent `tadpole.db` snapshot completed.
- [ ] **Health Check**: Staging `verify_all.py` must return 100% success.
- [ ] **Oversight**: Final sign-off from the project overseer.

---

## 🚀 Deployment SOP

### 1. The Blue-Green Strategy
- **Stage**: Spin up the new version of `server-rs` on a parallel port (e.g., 8001).
- **Traffic**: Run a "Warmlock" check: proxy a small percentage of dummy traffic to the new node and verify success rates via `telemetry.rs`.
- **Switch**: Update the load balancer or proxy to point all traffic to the new port 8001.

### 2. Live Verification
- **Pulse**: Monitor the global `Swarm Pulse` for any spike in error rates or latency.
- **Audit**: Verify that `merkle-chain` integrity is maintained post-deployment.

### 3. Cleanup
- **Action**: Shut down the "Blue" (old) instance after 15 minutes of stable operational telemetry.

---

## 🚦 Rollback Trigger
If `health_watchdog` detects a failure rate increase of >5% or if core endpoints (e.g., `/v1/agents`) return error statuses. Immediately revert the proxy to the "Blue" instance.

## 📊 Deployment Artifacts
Generate `reports/DEPLOYMENT_LOG_[VERSION]_[DATE].md`. Include all performance metrics from the first 5 minutes of live production traffic.