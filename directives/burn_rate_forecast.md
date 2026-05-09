> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[burn_rate_forecast]` in audit logs.
>
> ### AI Assist Note
> 📉 Directive: Burn Rate Forecast (SOP-FIN-01)
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 📉 Directive: Burn Rate Forecast (SOP-FIN-01)

## 🎯 Primary Objective
Deliver high-precision financial forecasting for the Tadpole OS cluster. Your goal is to predict the "Zero-Balance Date" based on current swarm activity and provider pricing tiers.

---

## 🏗️ Data Collection

### 1. Actual Consumption
- **Source**: `tadpole.db` -> `agents` table and `oversight_log`.
- **Action**: Aggregate `cost_usd` across all agents for the last 30 days.
- **Metric**: Calculate "Daily Average Burn."

### 2. Projected Activity
- **Context**: Review `feature_roadmap.md` and active `campaign_launch.md` directives.
- **Variable**: Estimate token multiplier for upcoming "High-Intensity" missions.

### 3. Pricing Tiers
- **Verification**: Check `Neural Registry` in the dashboard for current RPM/TPM rates per provider (Gemini, OpenAI, Groq).

---

## 📈 Forecasting Model

### 1. Base Forecast
- **Formula**: `(Current_Balance - Reserved_Buffer) / Daily_Average_Burn`
- **Output**: Days of Runway.

### 2. Swarm Expansion Scenario
- **Variable**: 2x Agent recruitment depth increase.
- **Output**: "Stress-Test" Runway.

---

## 🚦 Alerts & Triggers
- **WARNING**: Runway < 14 Days. Trigger `resource_allocation.md` optimization.
- **CRITICAL**: Runway < 3 Days. Trigger `emergency_shutdown.md` for non-essential sub-agents.

## 📝 Reporting Protocol
Generate a `FINANCIAL_PULSE_[DATE].md` report. Include a "Cost per Outcome" metric to identify the most expensive (and least efficient) specialized agents in the swarm.
[//]: # (Metadata: [burn_rate_forecast])
