# 🔋 Directive: Resource Allocation (SOP-STR-03)

## 🎯 Primary Objective
Deliver optimal resource distribution across the Tadpole OS cluster. This directive ensures that "High-Impact" missions receive priority access to specialized models, high-bandwidth tokens, and human overseer attention.

---

## 🏗️ Allocation Hubs

### 1. Token Economy
- **Tool**: `Budget Guard` and `metering.rs`.
- **Logic**: Allocate higher `budget_usd` limits to "Strategy" and "Security" agents. Restrict "Social" and "Drafting" agents to lower-cost local models (Ollama).
- **Enforcement**: Automatic mission suspension if an agent exceeds its assigned 24-hour quota.

### 2. Computational Prioritization
- **Check**: Server RAM and CPU pressure (see `ops_review.md`).
- **Action**: Use the `priority` flag in the mission registry to front-load critical system audits over non-essential background tasks.

### 3. Human Overseer Bandwidth
- **Constraint**: Identify missions that require frequent "Human-in-the-Loop" approvals.
- **Optimization**: Batch these approvals to minimize context-switching for the operator.

---

## 🛠️ Calibration SOP

### 1. Analysis
- **Source**: `burn_rate_forecast.md` and `engagement_report.md`.
- **Goal**: Identify under-utilized agents and re-allocate their budget to "High-Growth" sectors.

### 2. Reconfiguration
- **Action**: Update the `app_settings.json` and agent-specific `budget` fields in the SQLite database.

---

## 📊 Success Metrics
Track "Outcome-per-Dollar" and "Engine Pulse" (latency). Successful allocation is achieved when the cluster maintains 100% uptime for P0 missions while staying within 90% of the projected spend.