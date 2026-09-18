---
name: deploy-cycle
description: Standard procedure for deploying code from development to production.
---

# Deployment Protocol

The deployment cycle ensures code moves safely from a developer's machine to production users without causing downtime.

## Architecture

```mermaid
graph LR
    D[Dev] --> CI[CI Build]
    CI --> S[Staging]
    S --> E2E[E2E Tests]
    E2E --> M[Manual QA]
    M --> P[Production]
    P --> C[Canary]
    C --> F[Full Rollout]
```

### 1. CI Build (Continuous Integration)
Run unit tests, linting, and build validation on every commit.

### 2. Staging Deployment
Deploy build to a production-mirror environment. Connects to test databases.

### 3. Verification
- **Automated**: E2E tests run against staging.
- **Manual**: QA/Product team verifies the user experience.

### 4. Production Release
- **Canary**: Deploy to 1-5% of traffic. Monitor error rates.
- **Full Rollout**: Promote to 100% if Canary is healthy.

## When to Use
- **Feature Release**: Shipping new functionality.
- **Hotfix**: Patching a production bug.

## Operational Principles
1. **Automate Everything**: Humans make mistakes; scripts don't.
2. **Blue/Green Deployment**: Always have a way to switch back instantly.
3. **Observability**: Metrics must be visible *during* the deploy.
