---
name: deployment-procedures
description: Production deployment principles and decision-making. Safe deployment workflows, rollback strategies.
allowed-tools: Read, Glob, Grep, Bash
---

# Deployment Procedures

**Think before acting.**

## 1. Platform & Workflow
- **Vercel/Netlify**: Git push auto-deploy.
- **VPS/Docker**: Build -> Push -> Pull -> Restart.
- **Serverless**: Deploy function code.

## 2. The 5-Phase Process
1.  **PREPARE**: Verify code, env vars.
2.  **BACKUP**: Save state.
3.  **DEPLOY**: Execute & Watch.
4.  **VERIFY**: Health check, logs.
5.  **CONFIRM/ROLLBACK**: Stick or Twist.

---

## 🧠 Aletheia Reasoning Protocol (Release Engineering)

**Hope is not a strategy.**

### 1. Generator (Strategy Selection)
*   **Risk**: "Hotfix vs Major Release?".
*   **Method**: "Rolling vs Blue/Green vs Canary?".
*   **Fallback**: "Roll forward or backward?".

### 2. Verifier (Safety Checks)
*   **Pre-Flight**: "Migrations ran? Env vars set?".
*   **Capacity**: "Enough instances for rolling restart?".
*   **Observability**: "How will we know if it breaks?".

### 3. Reviser (Automation)
*   **Scripting**: "CI job > Manual".
*   **Hardening**: "Health check logic test".

---

## 🛡️ Security & Safety Protocol (Deployment)

1.  **Immutable Artifacts**: Deploy exactly what was tested.
2.  **Secrets**: Injected at runtime, never built-in.
3.  **Least Privilege**: Deployer needs minimal perms.
4.  **Gates**: Prod requires approval.

## Best Practices
- Small, frequent deploys.
- Feature flags.
- Test rollback first.
